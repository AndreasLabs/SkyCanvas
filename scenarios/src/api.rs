use conductor::redis::RedisConnection;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Scenario{
    async fn run(&mut self, t: f64, redis: Arc<Mutex<RedisConnection>>) -> Result<(), anyhow::Error>;
}

