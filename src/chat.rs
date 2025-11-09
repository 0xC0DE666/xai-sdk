//! Chat completion service client.
//!
//! Provides clients for chat completions, both blocking and streaming, as well as
//! utilities for processing and assembling streaming responses.

pub mod client {
    use crate::common;
    use crate::xai_api::chat_client::ChatClient;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new `ChatClient` connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ChatClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
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

    /// Creates a new `ChatClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `ChatClient<InterceptedService<Channel, impl Interceptor>>` - The connected client
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

/// Streaming utilities for chat completions.
///
/// Provides functions for processing streaming responses, assembling chunks into complete
/// responses, and flexible callback-based consumers for real-time token processing.
pub mod stream {
    use crate::xai_api::{
        Choice, CompletionMessage, FinishReason, GetChatCompletionChunk, GetChatCompletionResponse,
    };
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tonic::{Status, Streaming};

    /// Processes a streaming chat completion response, calling appropriate callbacks for each chunk.
    ///
    /// This function takes a gRPC streaming response containing `GetChatCompletionChunk` objects
    /// and processes each chunk as it arrives. It calls user-defined callbacks for content tokens,
    /// reasoning tokens, and complete chunks, allowing for real-time processing of AI responses.
    ///
    /// The token callbacks receive `(TokenContext, token: &str)` to properly handle
    /// multiple concurrent choices in a single stream.
    ///
    /// # Arguments
    /// * `stream` - The gRPC streaming response containing chat completion chunks
    /// * `consumer` - A [`Consumer`] that defines callback functions for handling different types of data
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
                    for choice in &chunk.choices {
                        if let Some(delta) = &choice.delta {
                            // Reasoning is complete when we have no reasoning content and either:
                            // - we have content (reasoning finished, content starting), or
                            // - the choice is finished (everything is done)
                            let reasoning_complete = delta.reasoning_content.is_empty()
                                && (!delta.content.is_empty()
                                    || choice.finish_reason != FinishReason::ReasonInvalid.into());

                            // Content is complete when the choice has finished
                            let content_complete =
                                choice.finish_reason != FinishReason::ReasonInvalid.into();

                            let token_context = TokenContext::init(
                                chunk.choices.len(),
                                choice.index as usize,
                                reasoning_complete,
                                content_complete,
                            );

                            if let Some(ref mut on_reason_token) = consumer.on_reason_token {
                                let reason_token = &delta.reasoning_content;
                                if !reason_token.is_empty() {
                                    on_reason_token(token_context.clone(), reason_token);
                                }
                            }

                            if let Some(ref mut on_content_token) = consumer.on_content_token {
                                let content_token = &delta.content;
                                if !content_token.is_empty() {
                                    on_content_token(token_context, content_token);
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
        tool_calls: Vec<crate::xai_api::ToolCall>,
        encrypted_content: String,
        finish_reason: i32,
        logprobs: Option<crate::xai_api::LogProbs>,
    }

    /// A callback-based consumer for processing streaming chat completion responses.
    ///
    /// The `Consumer` allows you to define custom callbacks that are invoked as chunks
    /// arrive from the stream. All callbacks are optional and can be set using builder
    /// methods or by using the pre-configured [`with_stdout()`](Consumer::with_stdout) or
    /// [`with_buffered_stdout()`](Consumer::with_buffered_stdout) constructors.
    ///
    /// # Callback Signatures
    /// - `on_content_token`: Called for each content token with `(TokenContext, token: &str)`
    /// - `on_reason_token`: Called for each reasoning token with `(TokenContext, token: &str)`
    /// - `on_chunk`: Called once per complete chunk received
    pub struct Consumer {
        /// Callback invoked for each content token in the stream.
        ///
        /// Receives `(TokenContext, token: &str)`
        pub on_content_token: Option<Box<dyn FnMut(TokenContext, &str) + Send + Sync>>,
        /// Callback invoked for each reasoning token in the stream.
        ///
        /// Receives `(TokenContext, token: &str)`
        pub on_reason_token: Option<Box<dyn FnMut(TokenContext, &str) + Send + Sync>>,
        /// Callback invoked once per complete chunk received.
        ///
        /// Receives `&GetChatCompletionChunk`
        pub on_chunk: Option<Box<dyn FnMut(&GetChatCompletionChunk) + Send + Sync>>,
    }

    impl Consumer {
        /// Create a new empty `Consumer` with no callbacks set.
        pub fn new() -> Self {
            Self {
                on_content_token: None,
                on_reason_token: None,
                on_chunk: None,
            }
        }

        /// Create a `Consumer` that prints content and reason tokens to stdout.
        ///
        /// This consumer only prints tokens from the first choice (index 0) to avoid
        /// output mangling when multiple choices are requested. For multi-choice
        /// streaming, use [`with_buffered_stdout()`](Consumer::with_buffered_stdout) instead.
        pub fn with_stdout() -> Self {
            Self {
                on_content_token: Some(Box::new(|token_context: TokenContext, token: &str| {
                    if token_context.choice_index != 0 {
                        return;
                    }

                    print!("{token}");
                    std::io::stdout().flush().expect("Error flushing stdout");

                    if token_context.content_complete {
                        println!("\n");
                    }
                })),
                on_reason_token: Some(Box::new(|token_context: TokenContext, token: &str| {
                    if token_context.choice_index != 0 {
                        return;
                    }

                    print!("{token}");
                    std::io::stdout().flush().expect("Error flushing stdout");

                    if token_context.reasoning_complete {
                        println!("\n");
                    }
                })),
                on_chunk: None,
            }
        }

        /// Create a Consumer that buffers tokens per choice and prints them in labeled blocks when each choice completes.
        ///
        /// This prevents output mangling when streaming multiple choices by buffering tokens
        /// until a choice finishes (has a finish_reason), then printing that choice's complete output
        /// in a clean labeled block.
        pub fn with_buffered_stdout() -> Self {
            #[derive(Default)]
            struct ChoiceBuffer {
                content: String,
                reasoning: String,
            }

            let buffers: Arc<Mutex<HashMap<i32, ChoiceBuffer>>> =
                Arc::new(Mutex::new(HashMap::new()));
            let finished: Arc<Mutex<HashMap<i32, bool>>> = Arc::new(Mutex::new(HashMap::new()));

            let buffers_content = buffers.clone();
            let buffers_reason = buffers.clone();
            let buffers_chunk = buffers.clone();
            let finished_clone = finished.clone();

            Self {
                on_content_token: Some(Box::new(
                    move |token_context: TokenContext, token: &str| {
                        let mut buffers = buffers_content.lock().unwrap();
                        let choice_buf = buffers
                            .entry(token_context.choice_index as i32)
                            .or_default();
                        choice_buf.content.push_str(token);
                    },
                )),
                on_reason_token: Some(Box::new(move |token_context: TokenContext, token: &str| {
                    let mut buffers = buffers_reason.lock().unwrap();
                    let choice_buf = buffers
                        .entry(token_context.choice_index as i32)
                        .or_default();
                    choice_buf.reasoning.push_str(token);
                })),
                on_chunk: Some(Box::new(move |chunk: &GetChatCompletionChunk| {
                    let mut buffers = buffers_chunk.lock().unwrap();
                    let mut finished = finished_clone.lock().unwrap();

                    for choice in &chunk.choices {
                        let idx = choice.index;

                        // Check if this choice just finished
                        if choice.finish_reason != 0 && !finished.contains_key(&idx) {
                            finished.insert(idx, true);

                            // Print the buffered content for this choice
                            if let Some(choice_buf) = buffers.remove(&idx) {
                                println!("\n--- Choice {idx} ---");
                                if !choice_buf.reasoning.is_empty() {
                                    println!("Reasoning:\n{}\n", choice_buf.reasoning);
                                }
                                if !choice_buf.content.is_empty() {
                                    println!("Content:\n{}\n", choice_buf.content);
                                }
                                println!("Finish reason: {}\n", choice.finish_reason);
                                std::io::stdout().flush().expect("Error flushing stdout");
                            }
                        }
                    }
                })),
            }
        }

        /// Builder method to set content token callback.
        ///
        /// The callback receives `(TokenContext, token: &str)` parameters.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(TokenContext, token: &str)`
        pub fn on_content_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(TokenContext, &str) + Send + Sync + 'static,
        {
            self.on_content_token = Some(Box::new(f));
            self
        }

        /// Builder method to set reason token callback.
        ///
        /// The callback receives `(TokenContext, token: &str)` parameters.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(TokenContext, token: &str)`
        pub fn on_reason_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(TokenContext, &str) + Send + Sync + 'static,
        {
            self.on_reason_token = Some(Box::new(f));
            self
        }

        /// Builder method to set chunk callback.
        ///
        /// Called once per chunk received, before token callbacks are invoked.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&GetChatCompletionChunk`
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

    /// Contextual information about a token in a streaming chat completion response.
    ///
    /// `TokenContext` provides metadata about the current token being processed, including
    /// which choice it belongs to, how many total choices are in the stream, and completion
    /// status flags for reasoning and content phases.
    ///
    /// This struct is passed to token callbacks (`on_content_token` and `on_reason_token`)
    /// to provide context about the token's position in the stream and the state of the
    /// response generation.
    ///
    #[derive(Clone, Debug)]
    pub struct TokenContext {
        /// The total number of choices in this streaming response.
        ///
        /// When `n > 1` is specified in the request, multiple choices are generated
        /// concurrently. This field indicates how many choices are being streamed.
        pub total_choices: usize,

        /// The index of the choice this token belongs to.
        ///
        /// Choices are indexed starting from 0. Use this to distinguish tokens from
        /// different choices when processing multi-choice streams.
        pub choice_index: usize,

        /// Indicates whether the reasoning phase is complete for this choice.
        ///
        /// This is `true` when:
        /// - The reasoning content has finished (no more reasoning tokens), and
        /// - Either content tokens have started appearing, or the choice has finished
        ///
        /// Use this flag to detect when the model has finished its reasoning phase
        /// and moved on to generating the final answer.
        pub reasoning_complete: bool,

        /// Indicates whether the content phase is complete for this choice.
        ///
        /// This is `true` when the choice has finished (i.e., `finish_reason != 0`),
        /// meaning no more tokens will be generated for this choice.
        ///
        /// Use this flag to detect when a choice has completed and perform any
        /// final processing or cleanup.
        pub content_complete: bool,
    }

    impl TokenContext {
        /// Creates a new `TokenContext` with the specified values.
        ///
        /// # Arguments
        ///
        /// * `total_choices` - The total number of choices in the stream
        /// * `choice_index` - The index of the choice this token belongs to
        /// * `reasoning_complete` - Whether the reasoning phase is complete
        /// * `content_complete` - Whether the content phase is complete
        ///
        /// # Returns
        ///
        /// A new `TokenContext` instance with the provided values.
        pub fn init(
            total_choices: usize,
            choice_index: usize,
            reasoning_complete: bool,
            content_complete: bool,
        ) -> Self {
            Self {
                total_choices,
                choice_index,
                reasoning_complete,
                content_complete,
            }
        }
    }
}
