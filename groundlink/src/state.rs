use std::collections::BTreeMap;
use std::sync::Arc;

use conductor::redis::{RedisConnection, RedisOptions};
use redis::RedisConnectionInfo;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct WSBridgeState {
    redis: Arc<Mutex<RedisConnection>>,
}

pub type StateHandle = std::sync::Arc<std::sync::Mutex<WSBridgeState>>;

impl Default for WSBridgeState {
    fn default() -> Self {
        Self {
            redis: Arc::new(Mutex::new(RedisConnection::new(
                RedisOptions::new(),
                "groundlink".to_string(),
            ))),
        }
    }
}

impl WSBridgeState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_handle(self) -> StateHandle {
        std::sync::Arc::new(std::sync::Mutex::new(self))
    }

    pub fn get_redis(&self) -> Arc<Mutex<RedisConnection>> {
        self.redis.clone()
    }
}
