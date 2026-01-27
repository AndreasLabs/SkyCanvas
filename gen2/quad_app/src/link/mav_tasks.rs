use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use log::info;

use crate::common::commands::QuadAppCommand;
use crate::link::mav_queues::MavlinkMessageType;
use crate::link::{mav_config::MavConfig, mav_queues::MavQueues};
use crate::link::tasks::MavTaskTrait;
use crate::common::context::QuadAppContext;

pub struct MavTasks {
    queues: MavQueues,
    enabled: AtomicBool,
    tasks: Vec<Box<dyn MavTaskTrait>>,
    context: QuadAppContext,
}

impl MavTasks{
    pub fn new(queues: MavQueues, context: QuadAppContext) -> Self {
        Self { queues, enabled: AtomicBool::new(false), tasks: Vec::new(), context }
    }

    pub fn add_task(&mut self, task: Box<dyn MavTaskTrait>) {
        self.tasks.push(task);
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        self.enabled.store(true, Ordering::Relaxed);
        info!("SkyCanvas // MavTasks // Starting");
        while self.enabled.load(Ordering::Relaxed) {
            self.tick()?;
            thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), anyhow::Error> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        // First read for for incoming messages
        let messages = self.queues.recv()?;
        if let Some(message) = messages {
            self.process_message(message)?;
        }
        let context = self.context.clone();
        let mut queues = self.queues.clone();
        // Then read for any commands from the app
        let commands = &mut self.context.commands.lock().unwrap();
        while let Some(command) = commands.pop_front() {
            let command_send = command.clone();
            self.process_command(&context, &mut queues, &command_send)?;
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

    fn process_command(&self, context: &QuadAppContext, queues: &mut MavQueues, command: &QuadAppCommand) -> Result<(), anyhow::Error> {
        // Tick each task w/ this command
        for task in self.tasks.iter() {
            task.handle_app_command(context, queues, command)?;
        }
        Ok(())
    }
}