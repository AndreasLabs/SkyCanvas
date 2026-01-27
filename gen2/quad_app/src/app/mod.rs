use std::{thread, time::Duration};

use log::{error, info};

use crate::{app::{app_config::AppConfig, systems::{AppSystemTrait, sys_mission_runner::SysMissionRunner, sys_waypoint::WaypointSystem}}, common::context::QuadAppContext};

pub mod systems;
pub mod missions;
pub mod patterns;
pub mod app_config;

pub struct QuadApp{
    
    pub config: AppConfig,
}

impl QuadApp{
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }


    pub fn start(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error> {
        info!("QuadApp // Starting");
        let context = context.clone();
        let app_thread_handle = std::thread::spawn(move || {


                let mut waypoint_system = WaypointSystem::new();
                let mut mission_runner = SysMissionRunner::new();

                waypoint_system.start(&context).unwrap();
                mission_runner.start(&context).unwrap();
            loop {
                let result = waypoint_system.tick(&context);
                let result = mission_runner.tick(&context);
              
                thread::sleep(Duration::from_millis(250));
            }
        });
        app_thread_handle.join().map_err(|e| anyhow::anyhow!("App thread panicked: {:?}", e))?;
        Ok(())
    }

}