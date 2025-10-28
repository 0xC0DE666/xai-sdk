use crate::{Choice, CompletionMessage, GetChatCompletionChunk, GetChatCompletionResponse};
use std::collections::HashMap;
use std::io::Write;
use tonic::{Status, Streaming};

/// Processes a streaming chat completion response, calling appropriate callbacks for each chunk.
///
/// This function takes a gRPC streaming response containing `GetChatCompletionChunk` objects
/// and processes each chunk as it arrives. It calls user-defined callbacks for content tokens,
/// reasoning tokens, and complete chunks, allowing for real-time processing of AI responses.
///
/// # Arguments
/// * `stream` - The gRPC streaming response containing chat completion chunks
/// * `consumer` - A `StreamConsumer` that defines callback functions for handling different types of data
///
/// # Returns
/// * `Ok(Vec<GetChatCompletionChunk>)` - All collected chunks from the stream
/// * `Err(Status)` - Any gRPC error that occurred during streaming
///
/// # Example
/// ```rust
/// let stream: Streaming<GetChatCompletionChunk> = response.into_inner();
/// let consumer = StreamConsumer::with_stdout();
/// let chunks = process_stream(stream, consumer).await?;
/// ```
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

/// Assembles a vector of streaming chunks into a complete GetChatCompletionResponse.
///
/// This function takes the collected chunks from a streaming chat completion response
/// and reconstructs them into a single, complete response object. It handles multiple
/// choices by grouping chunks by index, accumulates content from deltas, and preserves
/// all metadata and usage statistics.
///
/// # Arguments
/// * `chunks` - A vector of `GetChatCompletionChunk` objects collected from a streaming response
///
/// # Returns
/// * `Some(GetChatCompletionResponse)` - The complete assembled response if chunks are provided
/// * `None` - If the chunks vector is empty
///
/// # Features
/// - **Multiple Choices**: Groups chunks by choice index to handle multiple response choices
/// - **Content Accumulation**: Concatenates all delta content, reasoning, and encrypted content
/// - **Metadata Preservation**: Uses first chunk for consistent metadata, last chunk for final stats
/// - **Order Maintenance**: Sorts choices by index to preserve original order
///
/// # Example
/// ```rust
/// let chunks = process_stream(stream, consumer).await?;
/// if let Some(response) = assemble_response(chunks) {
///     println!("Complete response: {}", response.choices[0].message.as_ref().unwrap().content);
/// }
/// ```
pub fn assemble_response(chunks: Vec<GetChatCompletionChunk>) -> Option<GetChatCompletionResponse> {
    if chunks.is_empty() {
        return None;
    }

    // Use the first chunk for metadata that should be consistent across all chunks
    let first_chunk = &chunks[0];
    let last_chunk = &chunks[chunks.len() - 1];

    // Group chunks by choice index to handle multiple choices
    let mut choice_data: HashMap<i32, ChoiceData> = HashMap::new();

    for chunk in &chunks {
        for choice_chunk in &chunk.choices {
            let index = choice_chunk.index;
            let choice_data = choice_data.entry(index).or_insert_with(|| ChoiceData {
                content: String::new(),
                reasoning_content: String::new(),
                role: 0,
                tool_calls: Vec::new(),
                encrypted_content: String::new(),
                finish_reason: choice_chunk.finish_reason,
                logprobs: choice_chunk.logprobs.clone(),
            });

            // Accumulate content and reasoning from deltas
            if let Some(delta) = &choice_chunk.delta {
                choice_data.content.push_str(&delta.content);
                choice_data
                    .reasoning_content
                    .push_str(&delta.reasoning_content);
                choice_data
                    .encrypted_content
                    .push_str(&delta.encrypted_content);

                // Use the role from the delta (should be consistent)
                if delta.role != 0 {
                    choice_data.role = delta.role;
                }

                // Accumulate tool calls
                choice_data.tool_calls.extend(delta.tool_calls.clone());
            }

            // Update finish reason from the latest chunk
            choice_data.finish_reason = choice_chunk.finish_reason;
            choice_data.logprobs = choice_chunk.logprobs.clone();
        }
    }

    // Convert choice data to Choice objects
    let mut choices = Vec::new();
    for (index, data) in choice_data {
        let message = CompletionMessage {
            content: data.content,
            reasoning_content: data.reasoning_content,
            role: data.role,
            tool_calls: data.tool_calls,
            encrypted_content: data.encrypted_content,
        };

        choices.push(Choice {
            finish_reason: data.finish_reason,
            index,
            message: Some(message),
            logprobs: data.logprobs,
        });
    }

    // Sort choices by index to maintain order
    choices.sort_by_key(|c| c.index);

    // Use the last chunk's usage data (should have the final token counts)
    let usage = last_chunk.usage.clone();

    // Use the last chunk's citations (should be populated in the final chunk)
    let citations = last_chunk.citations.clone();

    Some(GetChatCompletionResponse {
        id: first_chunk.id.clone(),
        choices,
        created: first_chunk.created.clone(),
        model: first_chunk.model.clone(),
        system_fingerprint: first_chunk.system_fingerprint.clone(),
        usage,
        citations,
        settings: None, // Settings are not available in streaming responses
    })
}

/// Helper struct to accumulate data for each choice during assembly
#[derive(Default)]
struct ChoiceData {
    content: String,
    reasoning_content: String,
    role: i32,
    tool_calls: Vec<crate::ToolCall>,
    encrypted_content: String,
    finish_reason: i32,
    logprobs: Option<crate::LogProbs>,
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
