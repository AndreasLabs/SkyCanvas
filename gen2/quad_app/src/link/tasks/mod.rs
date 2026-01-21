use crate::{common::context::QuadAppContext, link::mav_queues::MavlinkMessageType};

pub trait MavTaskTrait{
    fn handle_mavlink_message(&self,context: &QuadAppContext, message: MavlinkMessageType) -> Result<(), anyhow::Error>;
   //fn handle_app_command(&self, command: QuadAppCommand) -> Result<(), anyhow::Error>;
}

pub mod mavtask_print;