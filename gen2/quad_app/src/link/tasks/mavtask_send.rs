use log::info;

use crate::{common::{commands::{QuadAppCommand, QuadAppCommandType}, context::QuadAppContext}, link::{mav_queues::MavQueues, tasks::MavTaskTrait}};



pub struct MavTaskSend{

}

impl MavTaskSend{
    pub fn new() -> Self {
        Self {}
    }
}

impl MavTaskTrait for MavTaskSend{  

    fn handle_app_command(&self, context: &QuadAppContext, queues: &mut MavQueues, command: &QuadAppCommand) -> Result<(), anyhow::Error>{
        match &command.cmd_type {
            QuadAppCommandType::MavlinkRaw(msg) => {
                info!("SkyCanvas // MavTaskSend // Sending message: {:#?}", msg);
                queues.send(msg.clone())?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
