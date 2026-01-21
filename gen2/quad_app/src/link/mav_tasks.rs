use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use log::info;

use crate::link::mav_queues::MavlinkMessageType;
use crate::link::{mav_config::MavConfig, mav_queues::MavQueues};
use crate::link::tasks::MavTaskTrait;
use crate::common::context::QuadAppContext;

pub struct MavTasks {
    config: MavConfig,
    queues: MavQueues,
    enabled: AtomicBool,
    tasks: Vec<Box<dyn MavTaskTrait>>,
    context: QuadAppContext,
}

impl MavTasks{
    pub fn new(config: MavConfig, queues: MavQueues, context: QuadAppContext) -> Self {
        Self { config, queues, enabled: AtomicBool::new(false), tasks: Vec::new(), context }
    }

    pub fn add_task(&mut self, task: Box<dyn MavTaskTrait>) {
        self.tasks.push(task);
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        self.enabled.store(true, Ordering::Relaxed);
        info!("SkyCanvas // MavTasks // Starting");
        while self.enabled.load(Ordering::Relaxed) {
            self.tick()?;
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), anyhow::Error> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        // First read for for incoming messages
        let messages = self.queues.recv()?;
        for message in messages {
            self.process_message(message)?;
        }

        
        Ok(())
    }

    fn process_message(&mut self, message: MavlinkMessageType) -> Result<(), anyhow::Error> {
        // Tick each task w/ this message
        for task in self.tasks.iter() {
            task.handle_mavlink_message(&self.context, message.clone())?;
        }
        Ok(())
    }
}