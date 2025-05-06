use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Error;

mod task;
pub mod examples;
pub use task::TransformerTask;

/// Transformer trait for message transformation
/// 
/// Implement this trait to create message transformers that:
/// 1. Listen to a specific Redis topic (get_topic)
/// 2. Transform the message content (transform)
/// 3. Publish to a specific output channel (get_out)
#[async_trait]
pub trait Transformer: Send + Sync + 'static {
    /// Get the output Redis channel
    fn get_out(&self) -> String;
    
    /// Get the input Redis topic to subscribe to
    fn get_topic(&self) -> String;
    
    /// Transform a message from JSON string to JSON string
    ///
    /// # Arguments
    ///
    /// * `message` - JSON string message content
    ///
    /// # Returns
    ///
    /// Transformed JSON string or error
    ///
    /// Note: For simple transformations that don't require await points,
    /// you can implement this method with synchronous code even though it's 
    /// defined as async.
    async fn transform(&self, message: String) -> Result<String, Error>;
} 