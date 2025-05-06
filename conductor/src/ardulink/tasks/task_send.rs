use crate::ardulink::cursed_strings;
use crate::{ardulink::connection::MavlinkConnection, redis::RedisConnection};
use crate::state::State;
use futures_util::StreamExt;
pub struct ArdulinkTask_Send{
    redis: Arc<Mutex<RedisConnection>>,
}

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use log::{debug, error, info, trace};
use mavlink::ardupilotmega::MavMessage;
use tokio::{task, time::{self, Duration}, task::JoinHandle};
use serde_json;
use redis::Commands;
use tokio::sync::Mutex;

impl ArdulinkTask_Send {
    pub async fn spawn(
        vehicle: MavlinkConnection,
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        info!("ArduLink // SendTask // Spawning + Connecting to Redis");
        let state = state.clone();
        task::spawn(async move {

            let redis = RedisConnection::new(state.redis.clone(), "ardulink_send".to_string());
            let (mut redis_sink, mut redis_stream) = redis.client.get_async_pubsub().await?.split();

            redis_sink.subscribe("channels/ardulink/send").await?;
                    
            info!("ArduLink // SendTask // Redis connected as ardulink_send");
            while !should_stop.load(Ordering::SeqCst) {
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }
                trace!("ArduLink // SendTask // Waiting for message");
                let msg = redis_stream.next().await.unwrap();
                let msg : String = msg.get_payload().unwrap();
                trace!("ArduLink // SendTask // Message received: {}", msg);
                let msg = serde_json::from_str::<MavMessage>(&msg)?;

                {
                    let vehicle = vehicle.lock().await;
                    let msg_type = cursed_strings::mavlink_message_type(&msg);
                    trace!("ArduLink // SendTask // Sending message: {}", msg_type);
                    vehicle.send(&mavlink::MavHeader::default(), &msg).unwrap();
                }
         
                // Check stop flag more frequently
                if should_stop.load(Ordering::SeqCst) {
                    info!("ArduLink // SendTask // Stopping");
                    break;
                }
         
            }
            info!("ArduLink // SendTask // Exiting");
            Ok(())
        })
    }
}
