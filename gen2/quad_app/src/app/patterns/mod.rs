use crate::common::{context::QuadAppContext, state::NED, waypoint::Waypoint};


#[derive(Default, Debug, Clone)]
pub struct PatternConfig{
    pub center_ned: NED,
    pub scale: f32,
    pub hold_time: f32,
}
impl PatternConfig{
    pub fn new(center_ned: NED, scale: f32, hold_time: f32) -> Self {
        Self { center_ned, scale, hold_time }
    }
}
pub trait QuadPatternTrait{
    fn generate(&mut self, context: &QuadAppContext, config: PatternConfig) -> Result<Vec<Waypoint>, anyhow::Error>;
}