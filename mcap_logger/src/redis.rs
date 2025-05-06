use log::{debug, error, info, trace};
use redis::Commands;

#[derive(Debug, Clone)]
pub struct RedisOptions {
    pub host: String,
    pub port: Option<u16>,
    pub password: Option<String>,
}

impl RedisOptions {
    pub fn new() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: None,
            password: None,
        }
    }

    pub fn to_redis_uri(&self) -> String {
        match self.port {
            Some(port) => format!("redis://{}:{}", self.host, port),
            None => format!("redis://{}", self.host),
        }
    }
}

pub struct RedisConnection {
    pub client: redis::Client,
    pub options: RedisOptions,
    pub client_name: String,
}

impl RedisConnection {
    pub fn new(options: RedisOptions, client_name: String) -> Self {
        let url = options.to_redis_uri();
        info!("Redis // {} // Starting with url: {}", client_name, url);
        let client = redis::Client::open(url).unwrap();
        info!("Redis // {} // Connected to Redis", client_name);
        Self {
            client,
            options,
            client_name,
        }
    }

    pub async fn subscribe_pattern(&mut self, pattern: &str) -> Result<redis::aio::PubSub, anyhow::Error> {
        let mut pubsub = self.client.get_async_pubsub().await?;
        info!("Redis // {} // Subscribing to pattern: {}", self.client_name, pattern);
        pubsub.psubscribe(pattern).await?;
        Ok(pubsub)
    }

    pub async fn subscribe(&mut self, channel: &str) -> Result<redis::aio::PubSub, anyhow::Error> {
        let mut pubsub = self.client.get_async_pubsub().await?;
        info!("Redis // {} // Subscribing to channel: {}", self.client_name, channel);
        pubsub.subscribe(channel).await?;
        Ok(pubsub)
    }
} 