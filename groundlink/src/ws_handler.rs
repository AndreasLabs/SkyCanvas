use crate::state::StateHandle;
use log::{debug, error, info};
use std::collections::BTreeMap;
use warp::ws::{Message, WebSocket};
use warp::{Rejection, Reply};
type Result<T> = std::result::Result<T, Rejection>;

use futures::{FutureExt, SinkExt, StreamExt};
use redis::Commands;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::mpsc;


#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum WSMessage {
    RedisSubscribe(String),
    RedisPublish(String, String),
    RedisUpdate(String, String),
}

pub async fn ws_handler(ws: warp::ws::Ws, state: StateHandle) -> Result<impl Reply> {
    Ok(ws.on_upgrade(|socket| async {
        ws_connect(socket, state).await;
    }))
}


pub async fn ws_connect(ws: WebSocket, state: StateHandle) {
    info!("New WebSocket connection");

    let (mut client_ws_sender, mut client_ws_rcv) = ws.split();

    
    let state_clone = state.clone();
    let start_time = std::time::Instant::now();

    // Create a shared list of channels to subscribe to
    let subscribed_channels = Arc::new(StdMutex::new(Vec::<String>::new()));
    let subscribed_channels_clone = subscribed_channels.clone();

    // Channel for Redis PubSub messages
    let (redis_tx, mut redis_rx) = mpsc::channel(100);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<WSMessage>(512);

    // Handle WebSocket -> Redis messages
    tokio::spawn(async move {
        let redis = {
            let state = state_clone.lock().unwrap();
            state.get_redis()
        };
                
        while let Some(msg) = rx.recv().await {
            match msg {
                WSMessage::RedisSubscribe(channel) => {
                    info!("Subscribing to Redis channel: {}", channel);
                    // Add to our tracked subscriptions
                    {
                        let mut channels = subscribed_channels.lock().unwrap();
                        if !channels.contains(&channel) {
                            channels.push(channel.clone());
                        }
                    } // Release the lock before async operations
                    
                    // Create a new PubSub connection immediately for this channel
                    let redis_mutex = redis.lock().await;
                    let redis_client = redis_mutex.client.clone();
                    drop(redis_mutex); // Release mutex before doing blocking operations
                    
                    // Clone needed values for the task
                    let redis_tx = redis_tx.clone();
                    let channel_clone = channel.clone();
                    
                    // Spawn a dedicated task for this subscription
                    tokio::task::spawn_blocking(move || {
                        match redis_client.get_connection() {
                            Ok(mut conn) => {
                                let mut pubsub = conn.as_pubsub();
                                if channel_clone.contains('*') {
                                    if let Err(e) = pubsub.psubscribe(&channel_clone) {
                                        error!("Failed to pattern subscribe to channel {}: {}", channel_clone, e);
                                        return;
                                    }
                                    info!("Pattern subscribed to channel: {}", channel_clone);
                                } else {
                                    if let Err(e) = pubsub.subscribe(&channel_clone) {
                                        error!("Failed to subscribe to channel {}: {}", channel_clone, e);
                                        return;
                                    }
                                    info!("Subscribed to channel: {}", channel_clone);
                                }
                                
                                info!("Starting dedicated listener for channel: {}", channel_clone);
                                loop {
                                    match pubsub.get_message() {
                                        Ok(msg) => {
                                            let channel = msg.get_channel_name().to_string();
                                            if let Ok(payload) = msg.get_payload::<String>() {
                                                info!("Redis message received: channel={}, payload={}", channel, payload);
                                                if let Err(e) = redis_tx.blocking_send((channel.clone(), payload.clone())) {
                                                    error!("Failed to send Redis message to WebSocket task: {}", e);
                                                    break;
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error!("Error getting Redis PubSub message: {}", e);
                                            std::thread::sleep(std::time::Duration::from_secs(1));
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to get Redis connection for subscription to {}: {}", channel_clone, e);
                            }
                        }
                    });
                },
                WSMessage::RedisPublish(channel, message) => {
                    info!("Publishing to Redis channel: {}", channel);
                    let redis_mutex = redis.lock().await;
                    let mut conn = match redis_mutex.client.get_connection() {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to get Redis connection for publish: {}", e);
                            continue;
                        }
                    };
                    if let Err(e) = conn.publish::<_, _, ()>(&channel, message) {
                        error!("Failed to publish to Redis channel {}: {}", channel, e);
                    }
                },
                WSMessage::RedisUpdate(key, value) => {
                    info!("Updating Redis key: {}", key);
                    let redis_mutex = redis.lock().await;
                    let mut conn = match redis_mutex.client.get_connection() {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to get Redis connection for update: {}", e);
                            continue;
                        }
                    };
                    if let Err(e) = conn.set::<_, _, ()>(&key, value) {
                        error!("Failed to update Redis key {}: {}", key, e);
                    }
                }
            }
        }
    });

    // No need for the separate Redis PubSub client in a separate thread - we now create dedicated ones per subscription

    // In the main async task, process WebSocket messages and forward Redis messages
    loop {
        tokio::select! {
            Some((channel, payload)) = redis_rx.recv() => {
                debug!("Forwarding Redis message from channel {} to WebSocket", channel);
                
                // Format the message as JSON with channel and content fields
                let formatted_message = serde_json::json!({
                    "channel": channel,
                    "content": payload
                });
                
                if let Err(e) = client_ws_sender.send(Message::text(serde_json::to_string(&formatted_message).unwrap())).await {
                    error!("Failed to forward Redis message to WebSocket: {}", e);
                    break;
                }
            },
            Some(result) = client_ws_rcv.next() => {
                match result {
                    Ok(msg) => {
                        if msg.is_close() {
                            info!("WebSocket client disconnected");
                            break;
                        }
                        
                        if msg.is_text() {
                            let text = msg.to_str().unwrap_or_default();
                            match serde_json::from_str::<WSMessage>(text) {
                                Ok(ws_msg) => {
                                    if let Err(e) = tx.send(ws_msg).await {
                                        error!("Failed to forward WebSocket message to Redis handler: {}", e);
                                        break;
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to deserialize WebSocket message: {}", e);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            },
            else => break,
        }
    }
    
    info!("WebSocket handler terminated");
}
