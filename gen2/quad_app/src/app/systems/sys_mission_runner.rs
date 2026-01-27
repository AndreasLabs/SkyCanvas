use crate::{app::{missions::{QuadMissionTrait, mission_hop::MissionHop}, systems::AppSystemTrait}, common::context::QuadAppContext};

pub struct SysMissionRunner{
    pub mission: Box<dyn QuadMissionTrait>,
}

impl SysMissionRunner{
    pub fn new() -> Self {
        Self { mission: Box::new(MissionHop::new()) }
    }
}

impl AppSystemTrait for SysMissionRunner{
    fn tick(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error> {
        self.mission.run(context)?;
        Ok(())
    }
}