mod ardulink;
mod cli_args;
mod commander;
mod transformers;

mod redis;
mod state;

use ardulink::config::ArdulinkConfig;
use redis::RedisOptions;
use clap::Parser;
use log::info;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use state::State;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_args::Args::parse();
    pretty_env_logger::init();
    info!("Starting conductor with config: {}", args.config);


    let ardulink_config = ArdulinkConfig{
        connection: ardulink::config::ArdulinkConnectionType::Tcp(String::from("127.0.0.1"), 5760),
    };
    let redis_options = RedisOptions{
        host: "127.0.0.1".to_string(),
        port: Some(6379),
        password: None,
    };
    let state = State::new(redis_options);
    
    // Create the transformers
    let transformers = transformers::examples::create_example_transformers();
    
    // Setup global stop flag
    let should_stop = Arc::new(AtomicBool::new(false));
    
    // Create ArduLink connection
    let mut ardulink = ardulink::connection::ArdulinkConnection::new(ardulink_config.connection, &state)?;
    
    // Add transformers to the connection
    ardulink.add_transformers(transformers);
    
    // Start ArduLink connection and transformer tasks
    ardulink.start_task().await?;

    // Handle ctrl+c to gracefully shutdown
    let should_stop_clone = should_stop.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("Received ctrl+c, shutting down...");
        should_stop_clone.store(true, Ordering::SeqCst);
    });

    loop { 
        // Check if we should stop
        if should_stop.load(Ordering::SeqCst) {
            break;
        }
        
        // Sleep for 100ms
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    // Stop tasks
    ardulink.stop_task().await?;
    
    info!("Conductor shutdown complete");
    Ok(())
}
