mod link;
mod app;
mod common;
use pretty_env_logger;

use crate::link::{QuadLink, mav_config::MavConfig};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    log::info!("SkyCanvas // Main // Starting");
    run()
}

fn run() -> Result<(), anyhow::Error> {
    let config = MavConfig::default();
    let mut quad_link = QuadLink::new(config.clone());
    
    match quad_link.start() {
        Ok(_) => {
            log::info!("SkyCanvas // Main // QuadLink started successfully");

            // busy loop
            loop {
                thread::sleep(Duration::from_secs(1));
            }
            Ok(())
        },
        Err(e) => {
            log::error!("SkyCanvas // Main // Error starting QuadLink: {}", e);
            Err(anyhow::anyhow!("Error starting QuadLink: {}", e))
        }
    }
}