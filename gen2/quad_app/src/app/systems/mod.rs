use crate::common::context::QuadAppContext;

pub mod sys_waypoint;
pub mod sys_mission_runner;

pub trait AppSystemTrait{
    fn tick(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error>;
}