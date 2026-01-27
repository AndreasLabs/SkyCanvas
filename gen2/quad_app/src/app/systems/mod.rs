use crate::common::context::QuadAppContext;

pub mod sys_waypoint;
pub mod sys_mission_runner;

pub trait AppSystemTrait{
    fn start(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error>;
    fn tick(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error>;
}