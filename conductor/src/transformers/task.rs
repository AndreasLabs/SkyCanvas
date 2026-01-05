use std::sync::Arc;
use anyhow::Error;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use redis::Commands;
use tokio::{task, task::JoinHandle, sync::Mutex, time::{self, Duration}};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{redis::RedisConnection, transformers::Transformer, state::State};

/// Task for managing message transformers
pub struct TransformerTask {
    transformers: Vec<Arc<dyn Transformer>>,
    redis: Arc<Mutex<RedisConnection>>,
    should_stop: Arc<AtomicBool>,
}

impl TransformerTask {
    /// Create a new transformer task
    ///
    /// # Arguments
    ///
    /// * `transformers` - Vector of transformer implementations
    /// * `redis` - Redis connection
    /// * `should_stop` - Atomic flag to signal stopping
    pub fn new(
        transformers: Vec<Arc<dyn Transformer>>,
        redis: Arc<Mutex<RedisConnection>>,
        should_stop: Arc<AtomicBool>,
    ) -> Self {
        Self {
            transformers,
            redis,
            should_stop,
        }
    }

    /// Spawn the transformer task
    ///
    /// # Arguments
    ///
    /// * `state` - Application state
    ///
    /// # Returns
    ///
    /// Join handle for the spawned task
    pub async fn spawn(
        transformers: Vec<Arc<dyn Transformer>>,
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), Error>> {
        info!("Transformers // TransformerTask // Spawning");

        let redis = RedisConnection::new(state.redis.clone(), "transformers".to_string());
        let redis = Arc::new(Mutex::new(redis));
        
        let task = Self::new(transformers, redis.clone(), should_stop.clone());
        
        task::spawn(async move {
            task.run().await
        })
    }

    /// Run the transformer task
    async fn run(&self) -> Result<(), Error> {
        info!("Transformers // TransformerTask // Starting");
        
        // Get subscription topics from all transformers
        let topics: Vec<String> = self.transformers
            .iter()
            .map(|t| t.get_topic())
            .collect();

        if topics.is_empty() {
            warn!("Transformers // TransformerTask // No transformers registered");
            return Ok(());
        }
        
        info!("Transformers // TransformerTask // Subscribing to topics: {:?}", topics);
        
        // Create a separate connection for publishing messages
        let publish_con = {
            let redis = self.redis.lock().await;
            redis.client.clone()
        };
        
        // Set up PubSub with async interface using the main connection
        let redis_con = self.redis.lock().await;
        let mut pubsub = redis_con.client.get_async_pubsub().await?;
        // Release the lock after getting pubsub
        drop(redis_con);
        
        // Subscribe to all topics
        for topic in &topics {
            pubsub.subscribe(topic).await?;
        }
        
        // Create message stream
        let mut stream = pubsub.into_on_message();
        
        // Process messages until should_stop is true
        while !self.should_stop.load(Ordering::SeqCst) {
            tokio::select! {
                Some(msg) = stream.next() => {
                    let channel: String = msg.get_channel()?;
                    let payload: String = msg.get_payload()?;
                    
                    debug!("Transformers // TransformerTask // Received message on {}", channel);
                    
                    // Find transformers that handle this topic
                    for transformer in &self.transformers {
                        if transformer.get_topic() == channel {
                            // Process with transformer
                            match transformer.transform(payload.clone()).await {
                                Ok(transformed) => {
                                    // Publish transformed message
                                    let output_channel = transformer.get_out();
                                    
                                    // Get a connection to Redis for publishing
                                    let mut con = publish_con.get_connection()?;
                                    
                                    // Publish the transformed message
                                    let publish_result: Result<(), redis::RedisError> = con.publish(&output_channel, &transformed);
                                    
                                    match publish_result {
                                        Ok(_) => {
                                            debug!("Transformers // TransformerTask // Published transformed message to {}", output_channel);
                                        },
                                        Err(e) => {
                                            error!("Transformers // TransformerTask // Failed to publish to {}: {}", output_channel, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Transformers // TransformerTask // Error transforming message: {}", e);
                                }
                            }
                        }
                    }
                }
                _ = time::sleep(Duration::from_millis(100)) => {
                    // Regular check of should_stop flag
                    if self.should_stop.load(Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
        
        info!("Transformers // TransformerTask // Stopping");
        Ok(())
    }
} 