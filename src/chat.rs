use crate::{Choice, CompletionMessage, GetChatCompletionChunk, GetChatCompletionResponse};
use std::io::Write;
use tonic::{Status, Streaming};

pub async fn process_stream(
    mut stream: Streaming<GetChatCompletionChunk>,
    mut consumer: StreamConsumer,
    // TODO: add proper error handling
) -> Result<GetChatCompletionResponse, Status> {
    let mut buf_reasoning_content = String::new();
    let mut buf_content = String::new();
    let mut complete_res = GetChatCompletionResponse::default();
    let mut init = true;
    let mut role: i32 = 0;
    let mut finish_reason: i32 = 0;

    // Process each chunk as it arrives
    loop {
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

                if init {
                    init = false;
                    complete_res.id = chunk.id;
                    complete_res.created = chunk.created;
                    complete_res.model = chunk.model;
                    complete_res.system_fingerprint = chunk.system_fingerprint;
                }

                if let Some(choice) = chunk.choices.last()
                    && role == 0
                {
                    if let Some(delta) = &choice.delta {
                        role = delta.role;
                    }
                }

                if chunk.usage.is_some() {
                    complete_res.usage = chunk.usage;
                }

                if chunk.citations.len() > 0 {
                    complete_res.citations = chunk.citations;
                }

                if let Some(choice) = chunk.choices.get(0) {
                    if let (Some(ref mut on_reason_token), Some(delta)) =
                        (consumer.on_reason_token.as_mut(), &choice.delta)
                    {
                        let reason_token = &delta.reasoning_content;
                        on_reason_token(reason_token);
                        buf_reasoning_content.push_str(reason_token);
                    }

                    if let (Some(ref mut on_content_token), Some(delta)) =
                        (consumer.on_content_token.as_mut(), &choice.delta)
                    {
                        let content_token = &delta.content;
                        on_content_token(content_token);
                        buf_content.push_str(content_token);
                    }

                    if choice.finish_reason > 0 {
                        finish_reason = choice.finish_reason;
                    }
                }
            }
            Err(status) => {
                return Err(status);
            }
        }
    }

    complete_res.choices.push(Choice {
        index: 0,
        message: Some(CompletionMessage {
            content: buf_content,
            reasoning_content: buf_reasoning_content,
            role: role,
            ..CompletionMessage::default()
        }),
        finish_reason: finish_reason,
        ..Choice::default()
    });

    Ok(complete_res)
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
