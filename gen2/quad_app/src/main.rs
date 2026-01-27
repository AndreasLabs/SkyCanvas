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

    let context_clone = context.clone();
    let quad_link_handle = thread::spawn(move || {
        match quad_link.start(&context_clone) {
            Ok(_) => {
                log::info!("SkyCanvas // Main // QuadLink started successfully");
            },
            Err(e) => {
                log::error!("SkyCanvas // Main // Error starting QuadLink: {}", e);
            }
        }
    });

    let context_clone = context.clone();
    let app_handle = thread::spawn(move || {
        info!("SkyCanvas // Main // Starting App");
        app.start(&context_clone)
    });

    // Wait for both threads to complete
    quad_link_handle.join().map_err(|e| anyhow::anyhow!("QuadLink thread panicked: {:?}", e))?;
    app_handle.join().map_err(|e| anyhow::anyhow!("App thread panicked: {:?}", e))??;
    
    Ok(())
}