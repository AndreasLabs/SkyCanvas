use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Foxglove WebSocket server channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub topic: String,
    pub encoding: String,
    pub schemaName: String,
    pub schema: serde_json::Value,
}

/// Foxglove WebSocket server protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "payload")]
pub enum ServerMessage {
    #[serde(rename = "advertise")]
    Advertise {
        channels: Vec<Channel>,
    },
    #[serde(rename = "unadvertise")]
    Unadvertise {
        channelIds: Vec<String>,
    },
    #[serde(rename = "message")]
    Message {
        channel: String,
        #[serde(rename = "logTime")]
        log_time: Option<i64>,
        #[serde(rename = "publishTime")]
        publish_time: Option<i64>,
        #[serde(rename = "receiveTime")]
        receive_time: i64,
        #[serde(rename = "message")]
        data: serde_json::Value,
    },
    #[serde(rename = "status")]
    Status {
        level: String,
        message: String,
    },
}

/// Foxglove WebSocket client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe {
        #[serde(rename = "channelId")]
        channel_id: String,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        #[serde(rename = "channelId")]
        channel_id: String,
    },
}

/// Schema generator for Redis JSON messages
pub struct SchemaGenerator;

impl SchemaGenerator {
    /// Generate a schema for a JSON message
    pub fn generate_schema(channel: &str, sample_message: &serde_json::Value) -> serde_json::Value {
        // Create a basic JSON schema based on message structure
        let mut properties = HashMap::new();
        
        // If the message is an object, extract its properties
        if let serde_json::Value::Object(obj) = sample_message {
            for (key, value) in obj {
                properties.insert(key.clone(), Self::get_type_for_value(value));
            }
        }
        
        // Create the schema
        serde_json::json!({
            "type": "object",
            "title": channel.replace("/", "."),
            "description": format!("Schema for {}", channel),
            "properties": properties,
        })
    }
    
    /// Get JSON schema type for a value
    fn get_type_for_value(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Null => serde_json::json!({"type": "null"}),
            serde_json::Value::Bool(_) => serde_json::json!({"type": "boolean"}),
            serde_json::Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    serde_json::json!({"type": "integer"})
                } else {
                    serde_json::json!({"type": "number"})
                }
            },
            serde_json::Value::String(_) => serde_json::json!({"type": "string"}),
            serde_json::Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    serde_json::json!({
                        "type": "array",
                        "items": Self::get_type_for_value(first)
                    })
                } else {
                    serde_json::json!({
                        "type": "array",
                        "items": {"type": "any"}
                    })
                }
            },
            serde_json::Value::Object(obj) => {
                let mut properties = HashMap::new();
                for (key, val) in obj {
                    properties.insert(key.clone(), Self::get_type_for_value(val));
                }
                serde_json::json!({
                    "type": "object",
                    "properties": properties
                })
            }
        }
    }
    
    /// Generate a unique channel ID
    pub fn generate_channel_id() -> String {
        Uuid::new_v4().to_string()
    }
} 