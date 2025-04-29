

use crate::redis::RedisOptions;


#[derive(Debug, Clone)]
pub struct State {
   pub redis: RedisOptions
}

impl State{
    pub fn new(redis: RedisOptions) -> Self{
        Self{redis}
    }
}
