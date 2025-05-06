use std::{sync::Arc, time::Duration};

use conductor::redis::{RedisConnection, RedisOptions};
use log::{debug, info};
use redis::RedisConnectionInfo;
use tokio::{sync::Mutex, time::Instant};

use crate::api::Scenario;

pub struct ScenarioRunner {
    pub current_t: f64,
    pub max_t: f64,
    pub scenario: Arc<Mutex<dyn Scenario>>,
    pub redis_handle: Arc<Mutex<RedisConnection>>,
    pub start_time: Instant,
}

impl ScenarioRunner {
    pub fn new(scenario: Arc<Mutex<dyn Scenario>>, max_t: f64, redis_info: RedisOptions) -> Self {
        let redis = RedisConnection::new(redis_info.clone(), "scenario".to_string());
        info!("Created with max_t: {} and redis: {:#?}", max_t, redis_info);
        Self {
            current_t: 0.0,
            max_t,
            scenario: scenario.clone(),
            redis_handle: Arc::new(Mutex::new(redis)),
            start_time: Instant::now()
        }
    }

    pub async fn run(&mut self) -> Result<(),anyhow::Error>{
        
        info!("Starting run");
        self.start_time = Instant::now();
        self.current_t = 0.0;

        while self.current_t < self.max_t{
            
            let mut scene = self.scenario.lock().await;
            scene.run(self.current_t,self.redis_handle.clone()  ).await?;
            if self.current_t % 1.0 == 0. {
                info!("T = {:0.1}s", self.current_t );
            }
            tokio::time::sleep(Duration::from_secs_f64(0.1)).await;
            let new_t = Instant::now().duration_since(self.start_time).as_secs_f64();
            let new_t = (new_t * 10. ).round() / 10.0;
            
            self.current_t = new_t;
        }

        info!("Scenario runner finished.");

        Ok(())
    }
}
