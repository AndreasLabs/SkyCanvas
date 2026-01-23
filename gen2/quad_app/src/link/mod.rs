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

use crate::{common::context::QuadAppContext, link::{mav_queues::MavQueues, tasks::{MavTaskTrait, mavtask_print::MavTaskPrint, mavtask_status_text::MavTaskStatusText}}};
pub struct QuadLink{


    queues: MavQueues,
    config: MavConfig,
}

impl QuadLink{
    pub fn new(config: MavConfig) -> Self {
        let queues = MavQueues::new();
  
        Self {
            queues,
            config,
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        info!("SkyCanvas // QuadLink // Starting");
        let context = QuadAppContext::new("quad_app".to_string());
        let config = self.config.clone();
        let queues = self.queues.clone();
        let context = context.clone();
        let io_handle = std::thread::spawn(move || {
            let mut io = MavIO::new(config.clone(), queues.clone());
            io.start()
        });

        let queues = self.queues.clone();
        let context = context.clone();
        let tasks_handle = std::thread::spawn(move || {
            let mut tasks = MavTasks::new(queues.clone(), context.clone());
           // tasks.add_task(Box::new(MavTaskPrint::new()));
            tasks.add_task(Box::new(MavTaskStatusText::new()));
            tasks.start()
    });

    io_handle.join().map_err(|e| anyhow::anyhow!("IO thread panicked: {:?}", e))??;
    tasks_handle.join().map_err(|e| anyhow::anyhow!("Tasks thread panicked: {:?}", e))??;
    Ok(())
  }
}