use log::info;

use crate::{common::context::QuadAppContext, link::{mav_queues::MavlinkMessageType, tasks::MavTaskTrait}};

pub struct MavTaskPrint{
 
}

impl MavTaskPrint{
    pub fn new() -> Self {
        Self {}
    }
}


impl MavTaskTrait for MavTaskPrint{

    fn handle_mavlink_message(&self,context: &QuadAppContext, message: MavlinkMessageType) -> Result<(), anyhow::Error> {
        info!("SkyCanvas // MavTaskPrint // Received message: {:#?}", message);
        Ok(())
    }
}