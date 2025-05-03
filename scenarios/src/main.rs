use std::sync::Arc;

use conductor::redis::RedisOptions;
use labs::scenario_lab_arm::ScenarioLabArm;
use runner::ScenarioRunner;
use tokio::sync::Mutex;

mod labs;
mod runner;
mod api;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
   pretty_env_logger::init();
   let scenario = ScenarioLabArm::default();
   let scenario = Arc::new(Mutex::new(scenario));

   let mut runner = ScenarioRunner::new(scenario, 30.0, RedisOptions::new());
   runner.run().await
}
