use crate::{ardulink::connection::MavlinkConnection, redis::RedisConnection};
use crate::state::State;
use futures_util::StreamExt;
use log::{debug, error, info, trace};
use mavlink::ardupilotmega::MavMessage;
use tokio::{task, time::{self, Duration}, task::JoinHandle};
use serde_json;
use redis::Commands;
use tokio::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};


pub struct ArdulinkTask_Heartbeat{
   
}

impl ArdulinkTask_Heartbeat {
    fn build_request_stream() -> mavlink::ardupilotmega::MavMessage {
        mavlink::ardupilotmega::MavMessage::REQUEST_DATA_STREAM(
            mavlink::ardupilotmega::REQUEST_DATA_STREAM_DATA {
                target_system: 0,
                target_component: 0,
                req_stream_id: 0,
                req_message_rate: 20,
                start_stop: 1,
            },
        )
    }

    pub async fn spawn(
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        info!("ArduLink // HeartbeatTask // Spawning");
        let state = state.clone();
        let heartbeat = MavMessage::HEARTBEAT(mavlink::ardupilotmega::HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: mavlink::ardupilotmega::MavType::MAV_TYPE_GCS,
            autopilot: mavlink::ardupilotmega::MavAutopilot::MAV_AUTOPILOT_INVALID,
            base_mode: mavlink::ardupilotmega::MavModeFlag::empty(),
            system_status: mavlink::ardupilotmega::MavState::MAV_STATE_ACTIVE,
            mavlink_version: 3,
        });
        task::spawn(async move {

            let mut redis = RedisConnection::new(state.redis.clone(), "ardulink_heartbeat".to_string());
            let (mut redis_sink, mut redis_stream) = redis.client.get_async_pubsub().await?.split();

            redis_sink.subscribe("channels/ardulink/recv").await?;
                    
            info!("ArduLink // HeartbeatTask // Redis connected as ardulink_heartbeat");

            info!("ArduLink // HeartbeatTask // Waiting for first heartbeat");
            while !should_stop.load(Ordering::SeqCst) {
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }

                let msg = redis_stream.next().await.unwrap();
                let msg : String = msg.get_payload().unwrap();
                let msg = serde_json::from_str::<MavMessage>(&msg)?;

                match msg {
                    MavMessage::HEARTBEAT(heartbeat) => {
                        info!("ArduLink // HeartbeatTask // Heartbeat received: {:?}", heartbeat);
                        break;
                    }
                    _ => {}
                }
            }

            info!("ArduLink // HeartbeatTask // First heartbeat received starting heartbeat loop");
            while !should_stop.load(Ordering::SeqCst) {
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }
                let hb_json = serde_json::to_string(&heartbeat).unwrap();
                let _: () = redis.client.publish("channels/ardulink/send", &hb_json).unwrap();
                time::sleep(Duration::from_millis(1000)).await;
            }
            debug!("ArduLink // RecvTask // Exiting");
            Ok(())
        })
    }
}
