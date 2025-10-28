use crate::GetChatCompletionChunk;
use std::io::Write;
use tonic::{Status, Streaming};

pub async fn process_stream(
    mut stream: Streaming<GetChatCompletionChunk>,
    mut consumer: StreamConsumer,
) -> Result<Vec<GetChatCompletionChunk>, Status> {
    let mut chunks: Vec<GetChatCompletionChunk> = Vec::new();

    loop {
        // Process each chunk as it arrives
        match stream.message().await {
            Ok(chunk) => {
                if chunk.is_none() {
                    // Stream complete
                    break;
                }

                // Handle the chunk
                let chunk = chunk.unwrap();

                if let Some(ref mut on_chunk) = consumer.on_chunk {
                    on_chunk(&chunk);
                }

                for choice in &chunk.choices {
                    if let (Some(ref mut on_reason_token), Some(delta)) =
                        (consumer.on_reason_token.as_mut(), &choice.delta)
                    {
                        let reason_token = &delta.reasoning_content;
                        on_reason_token(reason_token);
                    }

                    if let (Some(ref mut on_content_token), Some(delta)) =
                        (consumer.on_content_token.as_mut(), &choice.delta)
                    {
                        let content_token = &delta.content;
                        on_content_token(content_token);
                    }
                }
                chunks.push(chunk);
            }
            Err(status) => {
                return Err(status);
            }
        }
    }

    Ok(chunks)
}

// ####################
// STREAM CONSUMER
// ####################

pub struct StreamConsumer {
    pub on_content_token: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    pub on_reason_token: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    pub on_chunk: Option<Box<dyn FnMut(&GetChatCompletionChunk) + Send + Sync>>,
}

impl StreamConsumer {
    /// Create a new empty StreamConsumer
    pub fn new() -> Self {
        Self {
            on_content_token: None,
            on_reason_token: None,
            on_chunk: None,
        }
    }

    /// Create a StreamConsumer that prints content and reason tokens to stdout
    pub fn with_stdout() -> Self {
        Self {
            on_content_token: Some(Box::new(|s: &str| {
                print!("{s}");
                std::io::stdout().flush().expect("Error flushing stdout");
            })),
            on_reason_token: Some(Box::new(|s: &str| {
                print!("{s}");
                std::io::stdout().flush().expect("Error flushing stdout");
            })),
            on_chunk: None,
        }
    }

    /// Builder method to set content token callback
    pub fn on_content_token<F>(mut self, f: F) -> Self
    where
        F: FnMut(&str) + Send + Sync + 'static,
    {
        self.on_content_token = Some(Box::new(f));
        self
    }

    /// Builder method to set reason token callback
    pub fn on_reason_token<F>(mut self, f: F) -> Self
    where
        F: FnMut(&str) + Send + Sync + 'static,
    {
        self.on_reason_token = Some(Box::new(f));
        self
    }

    /// Builder method to set chunk callback
    pub fn on_chunk<F>(mut self, f: F) -> Self
    where
        F: FnMut(&GetChatCompletionChunk) + Send + Sync + 'static,
    {
        self.on_chunk = Some(Box::new(f));
        self
    }
}

impl Default for StreamConsumer {
    fn default() -> Self {
        Self::new()
    }
}
