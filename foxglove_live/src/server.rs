use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{
    accept_async,
    tungstenite::protocol::Message as WsMessage,
};

use crate::config::AppConfig;
use crate::redis_handler::RedisHandler;
use crate::schema::{ClientMessage, ServerMessage};

pub struct WebSocketServer {
    config: AppConfig,
    redis_handler: Arc<RedisHandler>,
    message_tx: broadcast::Sender<(String, serde_json::Value, i64)>,
    clients: Arc<Mutex<HashMap<String, ClientState>>>,
}

struct ClientState {
    id: String,
    subscriptions: HashSet<String>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub async fn new(config: AppConfig) -> Result<Self> {
        // Create broadcast channel for messages from Redis to WebSocket clients
        let (message_tx, _) = broadcast::channel(1000);
        
        // Create Redis handler
        let redis_handler = RedisHandler::new(
            config.redis(),
            config.channel_pattern.clone(),
            message_tx.clone(),
        )?;
        
        let redis_handler = Arc::new(redis_handler);
        
        // Start Redis handler
        redis_handler.start().await?;
        
        Ok(Self {
            config,
            redis_handler,
            message_tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Run the WebSocket server
    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.ws_host, self.config.ws_port);
        let socket_addr: SocketAddr = addr.parse()?;
        
        // Create TCP listener
        let listener = TcpListener::bind(&socket_addr).await?;
        info!("WebSocket server listening on {}", socket_addr);
        
        // Accept incoming connections
        while let Ok((stream, addr)) = listener.accept().await {
            info!("New WebSocket connection from: {}", addr);
            
            let client_id = uuid::Uuid::new_v4().to_string();
            
            // Store client state
            {
                let mut clients = self.clients.lock().await;
                clients.insert(client_id.clone(), ClientState {
                    id: client_id.clone(),
                    subscriptions: HashSet::new(),
                });
            }
            
            // Clone necessary references for the client handler
            let redis_handler = self.redis_handler.clone();
            let clients = self.clients.clone();
            let message_tx = self.message_tx.clone();
            let addr_clone = addr.clone();
            
            // Spawn a task to handle this client
            tokio::spawn(async move {
                match Self::handle_connection(stream, client_id.clone(), redis_handler, clients.clone(), message_tx).await {
                    Ok(_) => info!("WebSocket connection closed gracefully: {}", addr_clone),
                    Err(e) => error!("Error handling WebSocket connection: {}", e),
                }
                
                // Clean up client state when done
                let mut clients = clients.lock().await;
                clients.remove(&client_id);
                info!("WebSocket connection cleanup completed: {}", addr_clone);
            });
        }
        
        Ok(())
    }
    
    /// Handle a WebSocket connection
    async fn handle_connection(
        stream: TcpStream,
        client_id: String,
        redis_handler: Arc<RedisHandler>,
        clients: Arc<Mutex<HashMap<String, ClientState>>>,
        message_tx: broadcast::Sender<(String, serde_json::Value, i64)>,
    ) -> Result<()> {
        // Set TCP_NODELAY to improve latency
        if let Err(e) = stream.set_nodelay(true) {
            warn!("Failed to set TCP_NODELAY: {}", e);
        }
        
        // Accept WebSocket connection
        let ws_stream = match accept_async(stream).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to accept WebSocket connection: {}", e);
                return Err(anyhow!("WebSocket handshake failed: {}", e));
            }
        };
        
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send initial server status
        let status_msg = serde_json::to_string(&ServerMessage::Status {
            level: "info".to_string(),
            message: "Connected to Foxglove WebSocket Server".to_string(),
        })?;
        
        if let Err(e) = ws_sender.send(WsMessage::Text(status_msg)).await {
            error!("Failed to send initial status message: {}", e);
            return Err(anyhow!("Failed to send initial message: {}", e));
        }
        
        // Create subscription to Redis messages
        let mut message_rx = message_tx.subscribe();
        
        // Send initial channel advertisement
        if let Err(e) = Self::advertise_channels(&redis_handler, &mut ws_sender).await {
            error!("Failed to advertise initial channels: {}", e);
            return Err(e);
        }
        
        // Create channels for communication between tasks
        let (stop_tx, _) = tokio::sync::oneshot::channel();
        
        // Spawn a task to receive messages from Redis and forward to WebSocket
        let redis_to_ws_task = {
            let redis_handler = redis_handler.clone();
            let client_id = client_id.clone();
            let clients = clients.clone();
            let mut ws_sender = ws_sender.clone(); // Clone the sender for the task
            
            tokio::spawn(async move {
                debug!("Started Redis-to-WebSocket forwarder for client {}", client_id);
                
                while let Ok(msg) = message_rx.recv().await {
                    let (channel, data, timestamp) = msg;
                    
                    // Special "channel_update" message indicates that channels have changed
                    if channel == "channel_update" {
                        if let Err(e) = Self::advertise_channels(&redis_handler, &mut ws_sender).await {
                            error!("Failed to advertise channels: {}", e);
                            break;
                        }
                        continue;
                    }
                    
                    // Check if client is subscribed to this channel
                    let foxglove_channel_id_opt = {
                        // Use the public method get_channel_by_id to find the channel
                        match redis_handler.get_channels().await.iter()
                            .find(|c| c.topic == channel) {
                                Some(ch) => Some(ch.id.clone()),
                                None => None,
                            }
                    };
                    
                    // Only proceed if we found a valid Foxglove channel ID
                    if let Some(foxglove_channel_id) = foxglove_channel_id_opt {
                        // Check if client is subscribed to this channel
                        let is_subscribed = {
                            let clients = clients.lock().await;
                            if let Some(client) = clients.get(&client_id) {
                                client.subscriptions.contains(&foxglove_channel_id)
                            } else {
                                false
                            }
                        };
                        
                        // Only send if subscribed
                        if is_subscribed {
                            // Get the full channel info
                            if let Some(foxglove_channel) = redis_handler.get_channels().await.iter()
                                .find(|c| c.id == foxglove_channel_id) {
                                
                                // Construct Foxglove message
                                let message = ServerMessage::Message {
                                    channel: foxglove_channel.id.clone(),
                                    log_time: None,
                                    publish_time: None,
                                    receive_time: timestamp,
                                    data,
                                };
                                
                                // Send to WebSocket
                                match serde_json::to_string(&message) {
                                    Ok(json) => {
                                        let send_result = ws_sender.send(WsMessage::Text(json)).await;
                                        if let Err(err) = send_result {
                                            let err_str = err.to_string().to_lowercase();
                                            if err_str.contains("connection reset") {
                                                debug!("WebSocket connection reset: {}", err);
                                            } else {
                                                error!("Failed to send message to WebSocket: {}", err);
                                            }
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to serialize message: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                
                debug!("Redis to WebSocket task ended for client: {}", client_id);
            })
        };
        
        // Process incoming WebSocket messages
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    match msg {
                        WsMessage::Text(text) => {
                            // Parse client message
                            match serde_json::from_str::<ClientMessage>(&text) {
                                Ok(client_msg) => match client_msg {
                                    ClientMessage::Subscribe { channel_id } => {
                                        if let Err(e) = Self::handle_subscribe(
                                            client_id.clone(),
                                            channel_id,
                                            &clients,
                                        ).await {
                                            error!("Failed to handle subscribe: {}", e);
                                        }
                                    }
                                    ClientMessage::Unsubscribe { channel_id } => {
                                        if let Err(e) = Self::handle_unsubscribe(
                                            client_id.clone(),
                                            channel_id,
                                            &clients,
                                        ).await {
                                            error!("Failed to handle unsubscribe: {}", e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    warn!("Failed to parse client message: {}", e);
                                }
                            }
                        }
                        WsMessage::Close(_) => {
                            debug!("Received close message from client {}", client_id);
                            break;
                        }
                        WsMessage::Ping(data) => {
                            // Respond to ping with pong
                            if let Err(e) = ws_sender.send(WsMessage::Pong(data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        _ => {}  // Ignore other message types
                    }
                }
                Err(e) => {
                    // Handle tungstenite errors
                    match &e {
                        tokio_tungstenite::tungstenite::Error::ConnectionClosed => {
                            debug!("WebSocket connection closed normally");
                        }
                        tokio_tungstenite::tungstenite::Error::Protocol(_) => {
                            warn!("WebSocket protocol error: {}", e);
                        }
                        tokio_tungstenite::tungstenite::Error::Io(io_err) => {
                            if io_err.kind() == std::io::ErrorKind::ConnectionReset {
                                debug!("WebSocket connection reset by peer");
                            } else {
                                error!("WebSocket I/O error: {}", e);
                            }
                        }
                        _ => {
                            error!("WebSocket error: {}", e);
                        }
                    }
                    break;
                }
            }
        }
        
        // Clean up
        let _ = stop_tx.send(());
        let _ = redis_to_ws_task.await;
        
        Ok(())
    }
    
    /// Advertise available channels to the client
    async fn advertise_channels(
        redis_handler: &Arc<RedisHandler>,
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            WsMessage,
        >,
    ) -> Result<()> {
        let channels = redis_handler.get_channels().await;
        
        // Only advertise if we have channels
        if !channels.is_empty() {
            let channel_count = channels.len();
            let message = ServerMessage::Advertise {
                channels,
            };
            
            let json = serde_json::to_string(&message)?;
            ws_sender.send(WsMessage::Text(json)).await?;
            debug!("Advertised {} channels", channel_count);
        } else {
            debug!("No channels to advertise yet");
        }
        
        Ok(())
    }
    
    /// Handle a client subscription request
    async fn handle_subscribe(
        client_id: String,
        channel_id: String,
        clients: &Arc<Mutex<HashMap<String, ClientState>>>,
    ) -> Result<()> {
        let mut clients = clients.lock().await;
        
        if let Some(client) = clients.get_mut(&client_id) {
            client.subscriptions.insert(channel_id.clone());
            debug!("Client {} subscribed to channel {}", client_id, channel_id);
        } else {
            return Err(anyhow!("Client not found"));
        }
        
        Ok(())
    }
    
    /// Handle a client unsubscription request
    async fn handle_unsubscribe(
        client_id: String,
        channel_id: String,
        clients: &Arc<Mutex<HashMap<String, ClientState>>>,
    ) -> Result<()> {
        let mut clients = clients.lock().await;
        
        if let Some(client) = clients.get_mut(&client_id) {
            client.subscriptions.remove(&channel_id);
            debug!("Client {} unsubscribed from channel {}", client_id, channel_id);
        } else {
            return Err(anyhow!("Client not found"));
        }
        
        Ok(())
    }
} 