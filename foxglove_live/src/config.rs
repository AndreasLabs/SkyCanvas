use clap::Parser;
use conductor::redis::RedisOptions;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Foxglove WebSocket server for Redis messages")]
pub struct AppConfig {
    /// WebSocket server host
    #[arg(long, default_value = "0.0.0.0")]
    pub ws_host: String,

    /// WebSocket server port
    #[arg(long, default_value = "8765")]
    pub ws_port: u16,

    /// Redis server host
    #[arg(long, default_value = "127.0.0.1")]
    pub redis_host: String,

    /// Redis server port (optional)
    #[arg(long)]
    pub redis_port: Option<u16>,

    /// Redis password (optional)
    #[arg(long)]
    pub redis_password: Option<String>,

    /// Redis channel pattern to subscribe to
    #[arg(long, default_value = "*")]
    pub channel_pattern: String,
}

impl AppConfig {
    pub fn redis(&self) -> RedisOptions {
        RedisOptions {
            host: self.redis_host.clone(),
            port: self.redis_port,
            password: self.redis_password.clone(),
        }
    }
} 