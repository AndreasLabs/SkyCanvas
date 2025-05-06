use std::collections::HashMap;

use log::{debug, error, info, trace};
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
    pub async fn wait_for_message(&mut self, channel: &str, value: Option<String>) -> Result<(), anyhow::Error>{
        use futures_util::StreamExt; // Import StreamExt for .next()

        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe(channel).await?;
        let mut stream = pubsub.into_on_message();

        debug!("Redis // {} // Waiting for message on channel '{}'{}", self.client_name, channel,
            match &value {
                Some(v) => format!(" with value '{}'", v),
                None => "".to_string(),
            }
        );

        loop {
            match stream.next().await {
                Some(msg) => {
                    let payload: String = msg.get_payload()?;
                    debug!("Redis // {} // Received message on channel '{}': {}", self.client_name, channel, payload);

                    match &value {
                        Some(expected_value) => {
                            // Try to handle JSON string deserialization if needed
                            let parsed_payload = match serde_json::from_str::<String>(&payload) {
                                Ok(parsed) => parsed,
                                Err(_) => payload.clone(), // If not a JSON string, use as-is
                            };
                            
                            if parsed_payload.trim().to_ascii_uppercase() == expected_value.trim().to_ascii_uppercase() {
                                info!("Redis // {} // Received expected message on channel '{}'", self.client_name, channel);
                                return Ok(());
                            } else {
                                debug!("Redis // {} // Received unexpected message on channel '{}': {} (parsed: {})", 
                                       self.client_name, channel, payload, parsed_payload);
                            }
                            // else: continue waiting for the next message matching the value
                        }
                        None => {
                            // No specific value needed, first message is enough
                            info!("Redis // {} // Received first message on channel '{}'", self.client_name, channel);
                            return Ok(());
                        }
                    }
                }
                None => {
                    // Stream ended before the expected message was received
                    error!("Redis // {} // Stream for channel '{}' ended unexpectedly.", self.client_name, channel);
                    return Err(anyhow::anyhow!("Redis stream ended before the expected message was received on channel '{}'", channel));
                }
            }
        }
        // Note: The loop should only exit via return Ok(()) or return Err(...), so Ok(()) here is unreachable
        // but added for completeness if the loop logic were different. It's removed as unreachable.
    }
}