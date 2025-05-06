use anyhow::Result;
use conductor::redis::RedisOptions;
use futures_util::StreamExt;
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

use crate::schema::{Channel, SchemaGenerator};

pub struct RedisHandler {
    options: RedisOptions,
    client: redis::Client,
    channel_pattern: String,
    // Map of Redis channels to Foxglove channel IDs
    channels: Arc<Mutex<HashMap<String, Channel>>>,
    // Broadcast sender for messages to WebSocket clients
    message_tx: broadcast::Sender<(String, serde_json::Value, i64)>,
    // Set of sample messages for schema generation
    sample_messages: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl RedisHandler {
    /// Create a new Redis handler
    pub fn new(
        options: RedisOptions,
        channel_pattern: String,
        message_tx: broadcast::Sender<(String, serde_json::Value, i64)>,
    ) -> Result<Self> {
        let url = options.to_redis_uri();
        info!("Redis // Connecting to {}", url);
        let client = redis::Client::open(url)?;
        
        Ok(Self {
            options,
            client,
            channel_pattern,
            channels: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            sample_messages: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Get current timestamp in milliseconds
    fn current_time_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64
    }
    
    /// Start the Redis handler
    pub async fn start(&self) -> Result<JoinHandle<Result<()>>> {
        let client = self.client.clone();
        let channel_pattern = self.channel_pattern.clone();
        let message_tx = self.message_tx.clone();
        let channels = self.channels.clone();
        let sample_messages = self.sample_messages.clone();
        
        // Spawn a task to handle Redis messages
        let handle = tokio::spawn(async move {
            info!("Redis // Starting subscription to pattern: {}", channel_pattern);
            
            // Connect to Redis PubSub
            let mut pubsub = client.get_async_pubsub().await?;
            pubsub.psubscribe(&channel_pattern).await?;
            let mut stream = pubsub.on_message();
            
            info!("Redis // Subscribed to pattern: {}", channel_pattern);
            
            // Process messages
            while let Some(msg) = stream.next().await {
                let channel: String = msg.get_channel()?;
                let payload: String = msg.get_payload()?;
                
                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&payload) {
                    Ok(json_value) => {
                        // Use current time in milliseconds as timestamp
                        let timestamp = Self::current_time_millis();
                        
                        // Add to sample messages if not already present
                        let mut samples = sample_messages.lock().await;
                        if !samples.contains_key(&channel) {
                            debug!("Redis // New channel detected: {}", channel);
                            samples.insert(channel.clone(), json_value.clone());
                            
                            // Generate schema and create Foxglove channel
                            let schema = SchemaGenerator::generate_schema(&channel, &json_value);
                            let channel_id = SchemaGenerator::generate_channel_id();
                            
                            let foxglove_channel = Channel {
                                id: channel_id.clone(),
                                topic: channel.clone(),
                                encoding: "json".to_string(),
                                schemaName: format!("{}_schema", channel.replace("/", "_")),
                                schema,
                            };
                            
                            // Store channel mapping
                            channels.lock().await.insert(channel.clone(), foxglove_channel);
                            
                            // Notify for channel update - ignore errors if no subscribers
                            if message_tx.receiver_count() > 0 {
                                if let Err(e) = message_tx.send((
                                    "channel_update".to_string(),
                                    serde_json::Value::Null,
                                    timestamp,
                                )) {
                                    debug!("Failed to send channel update notification: {}", e);
                                }
                            }
                        }
                        
                        // Send message to WebSocket clients - only if there are subscribers
                        if message_tx.receiver_count() > 0 {
                            if let Err(e) = message_tx.send((channel, json_value, timestamp)) {
                                error!("Failed to send message to WebSocket clients: {}", e);
                            }
                        } else {
                            // No clients connected yet, this is normal
                            debug!("No WebSocket clients connected yet, skipping message for channel: {}", channel);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse message as JSON: {}", e);
                    }
                }
            }
            
            info!("Redis // Subscription stream ended");
            Ok(())
        });
        
        Ok(handle)
    }
    
    /// Get the current channels
    pub async fn get_channels(&self) -> Vec<Channel> {
        self.channels.lock().await.values().cloned().collect()
    }
    
    /// Get a specific channel by ID
    pub async fn get_channel_by_id(&self, id: &str) -> Option<Channel> {
        for channel in self.channels.lock().await.values() {
            if channel.id == id {
                return Some(channel.clone());
            }
        }
        None
    }
} 