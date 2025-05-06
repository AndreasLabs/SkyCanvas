use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use conductor::redis::RedisConnection;
use log::{debug, info};
use mavlink::ardupilotmega::{MavCmd, MavMessage, COMMAND_INT_DATA, COMMAND_LONG_DATA};
use redis::Commands;
use tokio::sync::Mutex;
use async_trait::async_trait;
use crate::api::Scenario;

pub struct ScenarioLabArm {
    health_verified: Option<f64>,
}

impl Default for ScenarioLabArm {
    fn default() -> Self {
        Self {
            health_verified: None,
        }
    }
}

#[async_trait]
impl Scenario for ScenarioLabArm{
    async fn run(&mut self, t: f64, redis: Arc<Mutex<RedisConnection>>) -> Result<(), anyhow::Error>{
        let mut redis = redis.lock().await;
        
        // Only check health status once
        if self.health_verified.is_none() {
            info!("Waiting for system to be HEALTHY before proceeding with arm scenario");
            redis.wait_for_message("ardulink/health/status", Some("HEALTHY".to_string())).await?;
            
            // Update the flag to avoid waiting again
            self.health_verified = Some(t);
            info!("System health verified as HEALTHY, proceeding with arm scenario");
        }
        let t = t - self.health_verified.unwrap();
      
        match t{
            1.0 => {
                let msg = MavMessage::COMMAND_LONG(COMMAND_LONG_DATA {
                    param1: 1.0,     // 1.0 to arm, 0.0 to disarm
                    param2: 21196.0, // 21196 is the code for arm/disarm forcefully
                    param3: 0.0,
                    param4: 0.0,
                    param5: 0.0,
                    param6: 0.0,
                    param7: 0.0,
                    command: MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
                    target_system: 0,
                    target_component: 0,
                    confirmation: 0,
                });
             
                info!("Sending arm command at t=25.0s!");
                redis.publish_mavlink_message("channels/ardulink/send", &msg)?;
            }

            _ => {}
        };
        Ok(())
    }
}