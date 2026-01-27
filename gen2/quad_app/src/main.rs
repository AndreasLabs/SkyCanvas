mod link;
mod app;
mod common;
use log::info;
use pretty_env_logger;

use crate::app::QuadApp;
use crate::app::app_config::AppConfig;
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
    let context = crate::common::context::QuadAppContext::new("quad_app".to_string());
    let app_config = AppConfig::new();
    let mut app = QuadApp::new(app_config);


    match quad_link.start(&context) {
        Ok(_) => {
            log::info!("SkyCanvas // Main // QuadLink started successfully");

        },
        Err(e) => {
            log::error!("SkyCanvas // Main // Error starting QuadLink: {}", e);

        }
    }  
    info!("SkyCanvas // Main // QuadLink started successfully, starting App");

    app.start(&context)?;
    Ok(())
}