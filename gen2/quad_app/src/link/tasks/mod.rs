use crate::{common::context::QuadAppContext, link::mav_queues::MavlinkMessageType};

pub trait MavTaskTrait{
    fn handle_mavlink_message(&self,context: &QuadAppContext, message: MavlinkMessageType) -> Result<(), anyhow::Error>;
   //fn handle_app_command(&self, command: QuadAppCommand) -> Result<(), anyhow::Error>;
}

pub mod mavtask_print;
pub mod mavtask_status_text;
pub mod mavtask_local_ned;
pub mod mavtask_lla;
pub mod mavtask_health;