use std::sync::{Arc, Mutex, RwLock};

use crate::common::state::QuadAppState;
use crate::common::commands::QuadAppCommand;
use crate::link::mav_queues::MavlinkMessageType;
#[derive(Clone)]
pub struct QuadAppContext{
    pub state: Arc<RwLock<QuadAppState>>,
    pub commands: Arc<Mutex<Vec<QuadAppCommand>>>,
}

impl QuadAppContext{
    pub fn new() -> Self {
        Self { state: Arc::new(RwLock::new(QuadAppState::new())), commands: Arc::new(Mutex::new(Vec::new())) }
    }
}