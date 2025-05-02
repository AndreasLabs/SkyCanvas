use crate::ardulink::cursed_strings;
use crate::{ardulink::connection::MavlinkConnection, redis::RedisConnection};
use crate::state::State;

pub struct ArdulinkTask_Recv{
    redis: Arc<Mutex<RedisConnection>>,
}

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use log::{debug, error, info, trace};
use tokio::{task, time::{self, Duration}, task::JoinHandle};
use serde_json;
use redis::Commands;
use tokio::sync::Mutex;

impl ArdulinkTask_Recv {
    pub async fn spawn(
        vehicle: MavlinkConnection,
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        info!("ArduLink // RecvTask // Spawning + Connecting to Redis");
        let redis = RedisConnection::new(state.redis.clone(), "ardulink_recv".to_string());
        let redis = Arc::new(Mutex::new(redis));
        info!("ArduLink // RecvTask // Redis connected as ardulink_recv");
        task::spawn(async move {
            
            while !should_stop.load(Ordering::SeqCst) {
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }
                let vehicle = vehicle.lock().await;
                // Use standard receive with a timeout by checking the flag frequently
                let recv_result = vehicle.recv();

                match recv_result {
                    Ok((_header, msg)) => {
                        // Process received message
                        let msg_json = serde_json::to_string(&msg).unwrap();
                        let msg_type = cursed_strings::mavlink_message_type(&msg);
                        let mut redis_conn = redis.lock().await;
                        let _: () = redis_conn.client.publish(format!("channels/ardulink/recv/{}", msg_type), &msg_json).unwrap();
                    }
                    Err(mavlink::error::MessageReadError::Io(e)) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            // No messages currently available to receive -- wait a while
                            time::sleep(Duration::from_millis(10)).await;
                        } else if !should_stop.load(Ordering::SeqCst) {
                            // Only log errors if we're not stopping
                            error!("ArduLink // RecvTask // Receive error: {e:?}");
                            break;
                        }
                    }
                    // Messages that didn't get through due to parser errors are ignored
                    _ => {}
                }

                // Check stop flag more frequently
                if should_stop.load(Ordering::SeqCst) {
                    info!("ArduLink // RecvTask // Stopping");
                    break;
                }
                
                // Allow other tasks to run
                task::yield_now().await;
            }
            debug!("ArduLink // RecvTask // Exiting");
            Ok(())
        })
    }
}
