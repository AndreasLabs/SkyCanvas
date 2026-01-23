use crate::common::context::QuadAppContext;





pub trait QuadMissionTrait{
    fn run(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error>;
}