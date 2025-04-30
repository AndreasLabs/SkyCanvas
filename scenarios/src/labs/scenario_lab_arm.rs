use conductor::redis::RedisConnection;

use crate::api::Scenario;




pub struct ScenarioLabArm {

}

impl Scenario for ScenarioLabArm{
    fn run(&mut self, t: f64, redis: &mut RedisConnection) -> Result<(), anyhow::Error>{


        Ok(())
    }
}