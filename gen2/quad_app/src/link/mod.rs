pub mod mav_io;
pub mod mav_tasks;
pub mod tasks;
pub mod mav_queues;
pub mod mav_config;

use mav_io::MavIO;
use mav_tasks::MavTasks;
use mav_config::MavConfig;
use mavlink::ardupilotmega::MavMessage;

use log::info;
use std::sync::mpsc;

use crate::link::mav_queues::MavQueues;
pub struct QuadLink{
    io: MavIO,

    queues: MavQueues,
    config: MavConfig,
}

impl QuadLink{
    pub fn new(config: MavConfig) -> Self {
        let queues = MavQueues::new();
        let io = MavIO::new(config.clone(), queues.clone());
       
        Self {
            io,

            queues,
            config,
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        info!("SkyCanvas // QuadLink // Starting");
        self.io.start()?;

        Ok(())
    }
}