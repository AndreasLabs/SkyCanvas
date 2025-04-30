use std::collections::HashMap;

use log::info;
use mavlink::ardupilotmega::MavMessage;
use redis::Commands;

#[derive(Debug, Clone)]
pub struct RedisOptions{
    pub host: String,
    pub port: Option<u16>,
    pub password: Option<String>,
}

impl RedisOptions{
    pub fn new() -> Self{
        Self{
            host: "127.0.0.1".to_string(),
            port: None,
            password: None,
        }
    }

    pub fn to_redis_uri(&self) -> String{
          match self.port{
            Some(port) => format!("redis://{}:{}", self.host, port),
            None => format!("redis://{}", self.host),
          }
    }
}

pub struct RedisConnection{
    pub client: redis::Client,
    pub options: RedisOptions,
    pub client_name: String,

}

impl RedisConnection{
    pub fn new(options: RedisOptions, client_name: String) -> Self{
        let url = options.to_redis_uri();
        info!("Redis // {} // Staring with url: {}", client_name, url);
        let client = redis::Client::open(url).unwrap();
        info!("Redis // {} // Connected to Redis", client_name);
        Self{
            client,
            options,
            client_name,
        }
    }

    pub fn publish_mavlink_message(&mut self, channel: &str, message: &MavMessage) -> Result<(), redis::RedisError>{
        let msg_json = serde_json::to_string(message)?;
        self.client.publish(channel, &msg_json)
    }
}