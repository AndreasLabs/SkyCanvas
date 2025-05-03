use conductor::redis::RedisConnection;
use log::info;
use mavlink::ardupilotmega::{MavCmd, MavMessage, COMMAND_INT_DATA, COMMAND_LONG_DATA};
use redis::Commands;

use crate::api::Scenario;



#[derive(Default)]
pub struct ScenarioLabArm {

}


impl Scenario for ScenarioLabArm{
    fn run(&mut self, t: f64, redis: &mut RedisConnection) -> Result<(), anyhow::Error>{

        match t{

            25.0 => {
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
             
                info!("Sending arm!");
                redis.publish_mavlink_message("channels/ardulink/send", &msg)?;
            }

            _ => {}
        };
        Ok(())
    }
}