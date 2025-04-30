use std::cmp::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::ardulink::connection::MavlinkConnection;
use crate::state::State;

pub struct ArdulinkTask_Send{
    redis: Arc<Mutex<RedisConnection>>,
}


