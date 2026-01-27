use crate::{common::{commands::QuadAppCommand, context::QuadAppContext}, link::mav_queues::{MavQueues, MavlinkMessageType}};

pub trait MavTaskTrait{
    fn handle_mavlink_message(&self,context: &QuadAppContext, message: MavlinkMessageType) -> Result<(), anyhow::Error>{
        Ok(())
    }
    fn handle_app_command(&self, context: &QuadAppContext, queues: &mut MavQueues, command: &QuadAppCommand) -> Result<(), anyhow::Error>{
        Ok(())
    }
}

pub mod mavtask_send;
pub mod mavtask_print;
pub mod mavtask_status_text;
pub mod mavtask_local_ned;
pub mod mavtask_lla;
pub mod mavtask_health;