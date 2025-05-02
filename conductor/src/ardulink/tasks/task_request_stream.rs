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


pub struct ArdulinkTask_RequestStream{
   
}

impl ArdulinkTask_RequestStream {


    pub async fn spawn(
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        info!("ArduLink // RequestStreamTask // Spawning");
        let state = state.clone();
        let request_stream = MavMessage::REQUEST_DATA_STREAM(mavlink::ardupilotmega::REQUEST_DATA_STREAM_DATA {
            target_system: 0,
            target_component: 0,
            req_stream_id: 0,
            req_message_rate: 2, // Recv Rate
            start_stop: 1,
        });

        task::spawn(async move {

            let mut redis = RedisConnection::new(state.redis.clone(), "ardulink_request_stream".to_string());
            let (mut redis_sink, mut redis_stream) = redis.client.get_async_pubsub().await?.split();

            redis_sink.subscribe("channels/ardulink/recv/HEARTBEAT").await?;
                    
            info!("ArduLink // RequestStreamTask // Redis connected as ardulink_request_stream");

            info!("ArduLink // RequestStreamTask // Waiting for first heartbeat");
            while !should_stop.load(Ordering::SeqCst) {
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }

                let msg = redis_stream.next().await.unwrap();
                let msg : String = msg.get_payload().unwrap();
                let msg = serde_json::from_str::<MavMessage>(&msg)?;

                match msg {
                    MavMessage::HEARTBEAT(heartbeat) => {
                        info!("ArduLink // RequestStreamTask // Heartbeat received: {:?}", heartbeat);
                        break;
                    }
                    _ => {}
                }
            }

            info!("ArduLink // RequestStreamTask // First heartbeat received starting request stream packet");
            let rs_json = serde_json::to_string(&request_stream).unwrap();
            
            let _: () = redis.client.publish("channels/ardulink/send", &rs_json).unwrap();
            debug!("ArduLink // RequestStreamTask // Exiting");
            Ok(())
        })
    }
}
