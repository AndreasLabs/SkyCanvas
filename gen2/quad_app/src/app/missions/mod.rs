use crate::common::context::QuadAppContext;



pub mod mission_hop;

pub trait QuadMissionTrait{
    fn run(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error>;
}