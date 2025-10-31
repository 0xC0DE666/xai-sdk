pub mod client {
    use crate::chat_client::ChatClient;
    use crate::common;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new ChatClient connected to the xAI API.
    ///
    /// # Returns
    /// * `Result<ChatClient<Channel>, tonic::transport::Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<ChatClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ChatClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> ChatClient<InterceptedService<Channel, impl Interceptor>> {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ChatClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ChatClient` using a provided interceptor.
    ///
    /// This mirrors [`new()`] channel creation but applies the custom interceptor
    /// instead of the default auth interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<ChatClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<ChatClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let client = ChatClient::with_interceptor(channel, interceptor);

        Ok(client)
    }

    /// Creates a new `ChatClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `ChatClient<InterceptedService<Channel, impl Interceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor,
    ) -> ChatClient<InterceptedService<Channel, impl Interceptor>> {
        ChatClient::with_interceptor(channel, interceptor)
    }
}

pub mod stream {
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
    pub async fn process(
        mut stream: Streaming<GetChatCompletionChunk>,
        mut consumer: Consumer,
    ) -> Result<Vec<GetChatCompletionChunk>, Status> {
        let mut chunks: Vec<GetChatCompletionChunk> = Vec::new();

        loop {
            // Process each chunk as it arrives
            match stream.message().await {
                Ok(chunk) => {
                    // Stream complete
                    if chunk.is_none() {
                        break;
                    }

                    // Handle the chunk
                    let chunk = chunk.unwrap();
                    if let Some(ref mut on_chunk) = consumer.on_chunk {
                        on_chunk(&chunk);
                    }

                    // Handle each choice
                    let total_choices = chunk.choices.len();
                    for choice in &chunk.choices {
                        if let Some(delta) = &choice.delta {
                            if let Some(ref mut on_reason_token) = consumer.on_reason_token {
                                let reason_token = &delta.reasoning_content;
                                if !reason_token.is_empty() {
                                    on_reason_token(
                                        total_choices,
                                        choice.index as usize,
                                        reason_token,
                                    );
                                }
                            }

                            if let Some(ref mut on_content_token) = consumer.on_content_token {
                                let content_token = &delta.content;
                                if !content_token.is_empty() {
                                    on_content_token(
                                        total_choices,
                                        choice.index as usize,
                                        content_token,
                                    );
                                }
                            }
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
    pub fn assemble(chunks: Vec<GetChatCompletionChunk>) -> Option<GetChatCompletionResponse> {
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

    pub struct Consumer {
        pub on_content_token: Option<Box<dyn FnMut(usize, usize, &str) + Send + Sync>>,
        pub on_reason_token: Option<Box<dyn FnMut(usize, usize, &str) + Send + Sync>>,
        pub on_chunk: Option<Box<dyn FnMut(&GetChatCompletionChunk) + Send + Sync>>,
    }

    impl Consumer {
        /// Create a new empty StreamConsumer
        pub fn new() -> Self {
            Self {
                on_content_token: None,
                on_reason_token: None,
                on_chunk: None,
            }
        }

        /// Create a Consumer that prints content and reason tokens to stdout
        pub fn with_stdout() -> Self {
            Self {
                on_content_token: Some(Box::new(
                    |total_choices: usize, choice_idx: usize, s: &str| {
                        print!("[choice {choice_idx}/{total_choices}] {s}");
                        std::io::stdout().flush().expect("Error flushing stdout");
                    },
                )),
                on_reason_token: Some(Box::new(
                    |total_choices: usize, choice_idx: usize, s: &str| {
                        print!("[choice {choice_idx}/{total_choices} reasoning] {s}");
                        std::io::stdout().flush().expect("Error flushing stdout");
                    },
                )),
                on_chunk: None,
            }
        }

        /// Builder method to set content token callback
        pub fn on_content_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(usize, usize, &str) + Send + Sync + 'static,
        {
            self.on_content_token = Some(Box::new(f));
            self
        }

        /// Builder method to set reason token callback
        pub fn on_reason_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(usize, usize, &str) + Send + Sync + 'static,
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

    impl Default for Consumer {
        fn default() -> Self {
            Self::new()
        }
    }
}
