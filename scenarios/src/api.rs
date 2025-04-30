use conductor::redis::RedisConnection;



pub trait Scenario{
    fn run(&mut self, t: f64, redis: &mut RedisConnection) -> Result<(), anyhow::Error>;
}

