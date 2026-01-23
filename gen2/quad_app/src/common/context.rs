use std::sync::{Arc, Mutex, RwLock};

use crate::common::commands::QuadAppCommand;
use crate::common::log_rerun::LogRerun;
use crate::common::state::QuadAppState;
#[derive(Clone)]
pub struct QuadAppContext {
    pub state: Arc<RwLock<QuadAppState>>,
    pub commands: Arc<Mutex<Vec<QuadAppCommand>>>,
    pub log_rerun: Arc<Mutex<LogRerun>>,
}

impl QuadAppContext {
    pub fn new(name: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(QuadAppState::new())),
            commands: Arc::new(Mutex::new(Vec::new())),
            log_rerun: Arc::new(Mutex::new(LogRerun::new(name))),
        }
    }
}
