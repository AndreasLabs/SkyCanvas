use crate::{app::missions::QuadMissionTrait, common::context::QuadAppContext};

pub struct MissionHop{
}

impl MissionHop{
    pub fn new() -> Self {
        Self {  }
    }
}

impl QuadMissionTrait for MissionHop{
    fn run(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error> {


        // Wait for quad health to be ok 
        let mut state = context.state.read().unwrap();
        match state.ekf_status.is_healthy() {
            Ok(_) => {
                log::info!("MissionHop // Quad health is ok");
            }
            Err(e) => {
                log::warn!("MissionHop // Waiting for quad health to be ok: {}", e);
                return Ok(());
            }
        }

        Ok(())
    }
}