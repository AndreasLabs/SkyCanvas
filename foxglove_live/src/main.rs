use clap::Parser;
use log::{info, error};
use std::sync::Arc;
use tokio::sync::Mutex;

mod server;
mod redis_handler;
mod config;
mod schema;

use config::AppConfig;
use server::WebSocketServer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "info,foxglove_live=debug");
    pretty_env_logger::init();
    
    // Parse command line arguments
    let config = AppConfig::parse();
    
    info!("Starting Foxglove WebSocket Server on {}:{}", config.ws_host, config.ws_port);
    info!("Connecting to Redis at {}", config.redis().to_redis_uri());
    
    // Create server instance
    let server = WebSocketServer::new(config.clone()).await?;
    let server = Arc::new(Mutex::new(server));
    
    // Run the server
    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.lock().await.run().await {
                error!("Server error: {}", e);
            }
        })
    };
    
    // Setup signal handler for graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        info!("Received Ctrl+C, shutting down...");
        let _ = tx.send(());
    });
    
    // Wait for shutdown signal
    let _ = rx.await;
    
    // Wait for server to finish
    let _ = server_handle.await;
    
    info!("Foxglove WebSocket Server shut down gracefully");
    Ok(())
}
