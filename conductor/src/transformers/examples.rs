use std::sync::Arc;
use anyhow::Error;
use async_trait::async_trait;
use log::{debug, info};
use mavlink::ardupilotmega::{MavMessage, STATUSTEXT_DATA};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::transformers::Transformer;

/// Example transformer for MAVLink STATUSTEXT messages
///
/// This transformer converts the numeric status text values in MAVLink STATUSTEXT
/// messages to human-readable ASCII strings.
pub struct StatusTextTransformer;

#[derive(Debug, Serialize, Deserialize)]
struct StatusTextData {
    #[serde(rename = "type")]
    message_type: String,
    severity: SeverityType,
    text: Vec<u8>,
    #[serde(flatten)]
    other: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SeverityType {
    #[serde(rename = "type")]
    severity_type: String,
}

/// Simple log message output format
#[derive(Debug, Serialize, Deserialize)]
struct LogMessage {
    text: String,
    severity: String,
    source: String,
}

#[async_trait]
impl Transformer for StatusTextTransformer {
    fn get_out(&self) -> String {
        "channels/ardulink/STATUSTEXT_STRING".to_string()
    }
    
    fn get_topic(&self) -> String {
        "channels/ardulink/recv/STATUSTEXT".to_string()
    }
    
    async fn transform(&self, message: String) -> Result<String, Error> {
        // Parse the input JSON
        let status_text: StatusTextData = serde_json::from_str(&message)?;
     
        // Convert the byte array to ASCII string, filtering out null bytes
        let text_string = status_text.text
            .iter()
            .take_while(|&&b| b != 0) // Stop at null terminator
            .map(|&b| b as char)
            .collect::<String>();
        
        debug!("StatusText: Converted {:?} -> {}", status_text.message_type, text_string);
        
        // Create a simple log message output
        let output = LogMessage {
            text: text_string,
            severity: status_text.severity.severity_type,
            source: "MAVLINK".to_string(),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&output)?;
        Ok(json)
    }
}

/// Create a new StatusTextTransformer
pub fn create_status_text_transformer() -> Arc<dyn Transformer> {
    Arc::new(StatusTextTransformer)
}

/// Example usage showing how to create a collection of transformers
pub fn create_example_transformers() -> Vec<Arc<dyn Transformer>> {
    vec![
        create_status_text_transformer(),
        // Add more transformers here as needed
    ]
} 