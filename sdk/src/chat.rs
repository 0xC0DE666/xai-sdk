//! Chat completion service client.
//!
//! Provides clients for chat completions, both blocking and streaming, as well as
//! utilities for processing and assembling streaming responses.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::chat_client::ChatClient as XChatClient;

    pub type ChatClient = XChatClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new `ChatClient` connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ChatClient, Error>` - The connected client or connection error
    ///
    pub async fn new(api_key: &str) -> Result<ChatClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XChatClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `ChatClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `ChatClient` - The connected client
    pub fn with_channel(channel: Channel, api_key: &str) -> ChatClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XChatClient::with_interceptor(channel, auth_intercept);

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
    /// * `Result<ChatClient, Error>` - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<ChatClient, Error> {
        let channel = common::channel::new().await?;
        let client = XChatClient::with_interceptor(channel, ClientInterceptor::new(interceptor));

        Ok(client)
    }

    /// Creates a new `ChatClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `ChatClient` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> ChatClient {
        XChatClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}

/// Streaming utilities for chat completions.
///
/// Provides functions for processing streaming responses, assembling chunks into complete
/// responses, and flexible callback-based consumers for real-time token processing.
pub mod stream {
    use crate::export::{Status, Streaming};
    use crate::xai_api::{
        CompletionMessage, CompletionOutput, FinishReason, GetChatCompletionChunk,
        GetChatCompletionResponse,
    };
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    /// Processes a streaming chat completion response, calling appropriate callbacks for each chunk.
    ///
    /// This function takes a gRPC streaming response containing `GetChatCompletionChunk` objects
    /// and processes each chunk as it arrives. It calls user-defined callbacks for content tokens,
    /// reasoning tokens, and complete chunks, allowing for real-time processing of AI responses.
    ///
    /// The token callbacks receive `(&OutputContext, token: &str)` to properly handle
    /// multiple concurrent outputs in a single stream.
    ///
    /// # Arguments
    /// * `stream` - The gRPC streaming response containing chat completion chunks
    /// * `consumer` - A [`Consumer`] that defines callback functions for handling different types of data.
    ///   The consumer's lifetime is inferred from its callbacks - if callbacks capture local variables,
    ///   the consumer must live at least as long as those variables.
    ///
    /// # Returns
    /// * `Ok(Vec<GetChatCompletionChunk>)` - All collected chunks from the stream
    /// * `Err(Status)` - Any gRPC error that occurred during streaming
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_sdk::chat::stream::{Consumer, process};
    ///
    /// // With static consumer
    /// let consumer = Consumer::with_stdout();
    /// // process(stream, consumer).await?;
    ///
    /// // With scoped consumer that captures local state
    /// let mut is_thinking = true;
    /// let consumer = Consumer::new()
    ///     .on_reasoning_complete(|_ctx| {
    ///         is_thinking = false;
    ///     });
    /// // process(stream, consumer).await?;
    /// ```
    pub async fn process(
        mut stream: Streaming<GetChatCompletionChunk>,
        mut consumer: Consumer<'_>,
    ) -> Result<Vec<GetChatCompletionChunk>, Status> {
        let mut chunks: Vec<GetChatCompletionChunk> = Vec::new();
        let mut reasoning_complete_flags: HashMap<i32, bool> = HashMap::new();
        let mut content_complete_flags: HashMap<i32, bool> = HashMap::new();
        let mut last_chunk: Option<GetChatCompletionChunk> = None;

        loop {
            // Process each chunk as it arrives
            match stream.message().await {
                Ok(chunk) => {
                    // Stream complete
                    let chunk = match chunk {
                        Some(c) => c,
                        None => break,
                    };

                    // Handle the chunk
                    if let Some(ref mut on_chunk) = consumer.on_chunk {
                        on_chunk(&chunk);
                    }

                    // Handle each output
                    for output in &chunk.outputs {
                        if let Some(delta) = &output.delta {
                            let reasoning_status = get_reasoning_status(
                                &delta.reasoning_content,
                                &delta.content,
                                output.finish_reason,
                            );

                            let content_status = get_content_status(
                                &delta.reasoning_content,
                                &delta.content,
                                output.finish_reason,
                            );

                            let output_ctx = OutputContext::new(
                                (output.index + 1) as usize,
                                output.index as usize,
                                reasoning_status.clone(),
                                content_status.clone(),
                            );

                            // Reasoning
                            if let Some(ref mut on_reason_token) = consumer.on_reason_token
                                && !delta.reasoning_content.is_empty()
                            {
                                on_reason_token(&output_ctx, &delta.reasoning_content);
                            }

                            if let Some(ref mut on_reasoning_complete) =
                                consumer.on_reasoning_complete
                            {
                                let was_complete = reasoning_complete_flags
                                    .get(&output.index)
                                    .copied()
                                    .unwrap_or(false);
                                if !was_complete && reasoning_status == PhaseStatus::Complete {
                                    on_reasoning_complete(&output_ctx);
                                    reasoning_complete_flags.insert(output.index, true);
                                }
                            }

                            // Content
                            if let Some(ref mut on_content_token) = consumer.on_content_token
                                && !delta.content.is_empty()
                            {
                                on_content_token(&output_ctx, &delta.content);
                            }

                            if let Some(ref mut on_content_complete) = consumer.on_content_complete
                            {
                                let was_complete = content_complete_flags
                                    .get(&output.index)
                                    .copied()
                                    .unwrap_or(false);
                                if !was_complete && content_status == PhaseStatus::Complete {
                                    on_content_complete(&output_ctx);
                                    content_complete_flags.insert(output.index, true);
                                }
                            }

                            // Inline citations
                            if let Some(ref mut on_inline_citations) = consumer.on_inline_citations
                                && !delta.citations.is_empty()
                            {
                                on_inline_citations(&output_ctx, &delta.citations);
                            }

                            // Tool calls
                            // server tool calls are usually first in the stream
                            // client tool calls are usually last in the stream
                            if let Some(ref mut on_tool_calls) = consumer.on_tool_calls
                                && !delta.tool_calls.is_empty()
                            {
                                on_tool_calls(&output_ctx, &delta.tool_calls);
                            }
                        }
                    }
                    last_chunk = Some(chunk.clone());
                    chunks.push(chunk);
                }
                Err(status) => {
                    return Err(status);
                }
            }
        }

        // Call usage and citations callbacks on the last chunk
        if let Some(ref last_chunk) = last_chunk {
            if let Some(ref mut on_usage) = consumer.on_usage {
                if let Some(ref usage) = last_chunk.usage {
                    on_usage(usage);
                }
            }

            if let Some(ref mut on_citations) = consumer.on_citations {
                if !last_chunk.citations.is_empty() {
                    on_citations(&last_chunk.citations);
                }
            }
        }

        Ok(chunks)
    }

    /// Determines the reasoning phase status based on the delta content and output finish reason.
    ///
    /// Reasoning is complete when:
    /// - We have no reasoning content AND either:
    ///   - We have content (reasoning finished, content starting), or
    ///   - The output is finished (everything is done)
    ///
    /// Reasoning is pending when:
    /// - We have reasoning content AND no content yet
    ///
    /// Otherwise, reasoning is in the initial state.
    fn get_reasoning_status(
        reasoning_content: &str,
        content: &str,
        finish_reason: i32,
    ) -> PhaseStatus {
        let has_reasoning = !reasoning_content.is_empty();
        let has_content = !content.is_empty();
        let is_finished = finish_reason != FinishReason::ReasonInvalid.into();

        if !has_reasoning && (has_content || is_finished) {
            PhaseStatus::Complete
        } else if has_reasoning && !has_content {
            PhaseStatus::Pending
        } else {
            PhaseStatus::Init
        }
    }

    /// Determines the content phase status based on the delta content and output finish reason.
    ///
    /// Content is complete when:
    /// - Reasoning is empty (reasoning phase is done) AND the output has finished
    ///
    /// Content is pending when:
    /// - We have content tokens being generated
    ///
    /// Otherwise, content is in the initial state.
    fn get_content_status(
        reasoning_content: &str,
        content: &str,
        finish_reason: i32,
    ) -> PhaseStatus {
        let has_reasoning = !reasoning_content.is_empty();
        let has_content = !content.is_empty();
        let is_finished = finish_reason != FinishReason::ReasonInvalid.into();

        if !has_reasoning && is_finished {
            PhaseStatus::Complete
        } else if has_content {
            PhaseStatus::Pending
        } else {
            PhaseStatus::Init
        }
    }

    /// Assembles a vector of streaming chunks into a complete GetChatCompletionResponse.
    ///
    /// This function takes the collected chunks from a streaming chat completion response
    /// and reconstructs them into a single, complete response object. It handles multiple
    /// outputs by grouping chunks by index, accumulates content from deltas, and preserves
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
    /// - **Multiple Outputs**: Groups chunks by output index to handle multiple response outputs
    /// - **Content Accumulation**: Concatenates all delta content, reasoning, and encrypted content
    /// - **Metadata Preservation**: Uses first chunk for consistent metadata, last chunk for final stats
    /// - **Order Maintenance**: Sorts outputs by index to preserve original order
    ///
    pub fn assemble(chunks: Vec<GetChatCompletionChunk>) -> Option<GetChatCompletionResponse> {
        if chunks.is_empty() {
            return None;
        }

        // Use the first chunk for metadata that should be consistent across all chunks
        let first_chunk = &chunks[0];
        let last_chunk = &chunks[chunks.len() - 1];

        // Group chunks by output index to handle multiple outputs
        let mut output_data: HashMap<i32, OutputData> = HashMap::new();

        for chunk in &chunks {
            for output_chunk in &chunk.outputs {
                let index = output_chunk.index;
                let output_data = output_data.entry(index).or_insert_with(|| OutputData {
                    content: String::new(),
                    reasoning_content: String::new(),
                    role: 0,
                    tool_calls: Vec::new(),
                    encrypted_content: String::new(),
                    citations: Vec::new(),
                    finish_reason: output_chunk.finish_reason,
                    logprobs: output_chunk.logprobs.clone(),
                });

                // Accumulate content and reasoning from deltas
                if let Some(delta) = &output_chunk.delta {
                    output_data.content.push_str(&delta.content);
                    output_data
                        .reasoning_content
                        .push_str(&delta.reasoning_content);
                    output_data
                        .encrypted_content
                        .push_str(&delta.encrypted_content);

                    // Use the role from the delta (should be consistent)
                    if delta.role != 0 {
                        output_data.role = delta.role;
                    }

                    // Accumulate tool calls
                    output_data.tool_calls.extend(delta.tool_calls.clone());

                    // Accumulate citations from delta
                    output_data.citations.extend(delta.citations.clone());
                }

                // Update finish reason from the latest chunk
                output_data.finish_reason = output_chunk.finish_reason;
                output_data.logprobs = output_chunk.logprobs.clone();
            }
        }

        // Convert output data to CompletionOutput objects
        let mut outputs = Vec::new();
        for (index, data) in output_data {
            let message = CompletionMessage {
                content: data.content,
                reasoning_content: data.reasoning_content,
                role: data.role,
                tool_calls: data.tool_calls,
                encrypted_content: data.encrypted_content,
                citations: data.citations,
            };

            outputs.push(CompletionOutput {
                finish_reason: data.finish_reason,
                index,
                message: Some(message),
                logprobs: data.logprobs,
            });
        }

        // Sort outputs by index to maintain order
        outputs.sort_by_key(|o| o.index);

        // Use the last chunk's usage data (should have the final token counts)
        let usage = last_chunk.usage.clone();

        // Use the last chunk's citations (should be populated in the final chunk)
        let citations = last_chunk.citations.clone();

        Some(GetChatCompletionResponse {
            id: first_chunk.id.clone(),
            outputs,
            created: first_chunk.created.clone(),
            model: first_chunk.model.clone(),
            system_fingerprint: first_chunk.system_fingerprint.clone(),
            usage,
            citations,
            settings: None,     // Settings are not available in streaming responses
            debug_output: None, // Debug output is not available in streaming responses
        })
    }

    /// Helper struct to accumulate data for each output during assembly
    #[derive(Default)]
    struct OutputData {
        content: String,
        reasoning_content: String,
        role: i32,
        tool_calls: Vec<crate::xai_api::ToolCall>,
        encrypted_content: String,
        citations: Vec<crate::xai_api::InlineCitation>,
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
    /// # Lifetime
    ///
    /// The lifetime parameter `'a` is inferred by the compiler based on how the `Consumer` is used.
    /// If your callbacks capture local variables, the lifetime will be scoped to that context.
    /// If your callbacks only capture owned data or have no captures, the lifetime can be `'static`,
    /// allowing the consumer to be stored anywhere for the duration of the program.
    ///
    /// # Callback Signatures (ordered by execution order)
    /// - `on_chunk`: Called once per complete chunk received
    /// - `on_reason_token`: Called for each reasoning token with `(&OutputContext, token: &str)`
    /// - `on_reasoning_complete`: Called once when reasoning phase completes for an output with `&OutputContext`
    /// - `on_content_token`: Called for each content token with `(&OutputContext, token: &str)`
    /// - `on_content_complete`: Called once when content phase completes for an output with `&OutputContext`
    /// - `on_inline_citations`: Called when inline citations are present in a delta with `(&OutputContext, &[InlineCitation])`
    /// - `on_tool_calls`: Called when tool calls are present in the last delta per output with `(&OutputContext, &[ToolCall])`
    /// - `on_usage`: Called once on the last chunk with `&SamplingUsage`
    /// - `on_citations`: Called once on the last chunk with `&[String]` (citation URLs)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_sdk::chat::stream::Consumer;
    ///
    /// // Consumer with static lifetime (no local captures)
    /// let consumer = Consumer::with_stdout();
    ///
    /// // Consumer that captures local variables (scoped lifetime)
    /// let mut is_thinking = true;
    /// let consumer = Consumer::new()
    ///     .on_reasoning_complete(|_ctx| {
    ///         is_thinking = false;
    ///     });
    /// ```
    pub struct Consumer<'a> {
        /// Callback invoked once per complete chunk received.
        ///
        /// Receives `&GetChatCompletionChunk`
        pub on_chunk: Option<Box<dyn FnMut(&GetChatCompletionChunk) + Send + Sync + 'a>>,

        /// Callback invoked for each reasoning token in the stream.
        ///
        /// Receives `(&OutputContext, token: &str)`
        pub on_reason_token: Option<Box<dyn FnMut(&OutputContext, &str) + Send + Sync + 'a>>,

        /// Callback invoked once when the reasoning phase completes for an output.
        ///
        /// This callback is called only once per output when the reasoning phase transitions
        /// to `Complete`. Useful for performing cleanup or formatting when reasoning finishes.
        ///
        /// Receives `&OutputContext` with information about which output completed.
        pub on_reasoning_complete: Option<Box<dyn FnMut(&OutputContext) + Send + Sync + 'a>>,

        /// Callback invoked for each content token in the stream.
        ///
        /// Receives `(&OutputContext, token: &str)`
        pub on_content_token: Option<Box<dyn FnMut(&OutputContext, &str) + Send + Sync + 'a>>,

        /// Callback invoked once when the content phase completes for an output.
        ///
        /// This callback is called only once per output when the content phase transitions
        /// to `Complete`. Useful for performing cleanup or formatting when content generation finishes.
        ///
        /// Receives `&OutputContext` with information about which output completed.
        pub on_content_complete: Option<Box<dyn FnMut(&OutputContext) + Send + Sync + 'a>>,

        /// Callback invoked when inline citations are present in a delta.
        ///
        /// This callback is called whenever a delta contains inline citations for an output.
        ///
        /// Receives `(&OutputContext, &[InlineCitation])` with the output context and citations.
        pub on_inline_citations: Option<
            Box<dyn FnMut(&OutputContext, &[crate::xai_api::InlineCitation]) + Send + Sync + 'a>,
        >,

        /// Callback invoked when tool calls are present in the last delta per output.
        ///
        /// This callback is called when tool calls appear in a delta, typically in the last delta
        /// for an output before it completes.
        ///
        /// Receives `(&OutputContext, &[ToolCall])` with the output context and tool calls.
        pub on_tool_calls:
            Option<Box<dyn FnMut(&OutputContext, &[crate::xai_api::ToolCall]) + Send + Sync + 'a>>,

        /// Callback invoked once on the last chunk with usage statistics.
        ///
        /// This callback is called once when the stream completes, providing final usage statistics.
        ///
        /// Receives `&SamplingUsage` with token usage information.
        pub on_usage: Option<Box<dyn FnMut(&crate::xai_api::SamplingUsage) + Send + Sync + 'a>>,

        /// Callback invoked once on the last chunk with citations.
        ///
        /// This callback is called once when the stream completes, providing all citations
        /// from the final chunk.
        ///
        /// Receives `&[String]` with all citation URLs from the last chunk.
        pub on_citations: Option<Box<dyn FnMut(&[String]) + Send + Sync + 'a>>,
    }

    impl<'a> Consumer<'a> {
        /// Create a new empty `Consumer` with no callbacks set.
        ///
        /// The lifetime `'a` will be inferred from the callbacks you add using the builder methods.
        /// If you add callbacks that capture local variables, the consumer will be scoped to that context.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use xai_sdk::chat::stream::Consumer;
        ///
        /// // Empty consumer
        /// let consumer = Consumer::new();
        ///
        /// // Consumer with callbacks that capture local state
        /// let mut counter = 0;
        /// let consumer = Consumer::new()
        ///     .on_content_token(|_ctx, _token| {
        ///         counter += 1;
        ///     });
        /// ```
        pub fn new() -> Self {
            Self {
                on_chunk: None,
                on_reason_token: None,
                on_reasoning_complete: None,
                on_content_token: None,
                on_content_complete: None,
                on_inline_citations: None,
                on_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Create a `Consumer` that prints content and reason tokens to stdout.
        ///
        /// This consumer only prints tokens from the first output (index 0) to avoid
        /// output mangling when multiple outputs are requested. For multi-output
        /// streaming, use [`with_buffered_stdout()`](Consumer::with_buffered_stdout) instead.
        ///
        /// The returned consumer has a `'static` lifetime since it doesn't capture any local variables.
        /// You can still add additional callbacks using builder methods, but those callbacks must also
        /// be `'static` (not capture local variables).
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use xai_sdk::chat::stream::Consumer;
        ///
        /// // Basic usage
        /// let consumer = Consumer::with_stdout();
        ///
        /// // Can still add static callbacks
        /// let consumer = Consumer::with_stdout()
        ///     .on_usage(|usage| {
        ///         println!("Tokens used: {}", usage.total_tokens);
        ///     });
        /// ```
        pub fn with_stdout() -> Self {
            Self {
                on_chunk: None,
                on_reason_token: Some(Box::new(|ctx: &OutputContext, token: &str| {
                    if ctx.output_index != 0 {
                        return;
                    }

                    print!("{token}");
                    std::io::stdout().flush().expect("Error flushing stdout");
                })),
                on_reasoning_complete: Some(Box::new(|_ctx: &OutputContext| {
                    println!("\n");
                })),
                on_content_token: Some(Box::new(|ctx: &OutputContext, token: &str| {
                    if ctx.output_index != 0 {
                        return;
                    }

                    print!("{token}");
                    std::io::stdout().flush().expect("Error flushing stdout");
                })),
                on_content_complete: Some(Box::new(|_ctx: &OutputContext| {
                    println!("\n");
                })),
                on_inline_citations: None,
                on_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Create a Consumer that buffers tokens per output and prints them in labeled blocks when each output completes.
        ///
        /// This prevents output mangling when streaming multiple outputs by buffering tokens
        /// until an output finishes (has a finish_reason), then printing that output's complete content
        /// in a clean labeled block.
        ///
        /// The returned consumer has a `'static` lifetime since it doesn't capture any local variables.
        /// You can still add additional callbacks using builder methods, but those callbacks must also
        /// be `'static` (not capture local variables).
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use xai_sdk::chat::stream::Consumer;
        ///
        /// // For multiple outputs, this prevents interleaved output
        /// let consumer = Consumer::with_buffered_stdout();
        /// ```
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
                on_chunk: Some(Box::new(move |chunk: &GetChatCompletionChunk| {
                    let mut buffers = buffers_chunk.lock().unwrap();
                    let mut finished = finished_clone.lock().unwrap();

                    for output in &chunk.outputs {
                        let idx = output.index;

                        // Check if this output just finished
                        if output.finish_reason != 0 && !finished.contains_key(&idx) {
                            finished.insert(idx, true);

                            // Print the buffered content for this output
                            if let Some(output_buf) = buffers.remove(&idx) {
                                println!("\n--- Output {idx} ---");
                                if !output_buf.reasoning.is_empty() {
                                    println!("Reasoning:\n{}\n", output_buf.reasoning);
                                }
                                if !output_buf.content.is_empty() {
                                    println!("Content:\n{}\n", output_buf.content);
                                }
                                println!("Finish reason: {}\n", output.finish_reason);
                                std::io::stdout().flush().expect("Error flushing stdout");
                            }
                        }
                    }
                })),
                on_reason_token: Some(Box::new(move |ctx: &OutputContext, token: &str| {
                    let mut buffers = buffers_reason.lock().unwrap();
                    let output_buf = buffers.entry(ctx.output_index as i32).or_default();
                    output_buf.reasoning.push_str(token);
                })),
                on_reasoning_complete: None,
                on_content_token: Some(Box::new(move |ctx: &OutputContext, token: &str| {
                    let mut buffers = buffers_content.lock().unwrap();
                    let output_buf = buffers.entry(ctx.output_index as i32).or_default();
                    output_buf.content.push_str(token);
                })),
                on_content_complete: None,
                on_inline_citations: None,
                on_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Builder method to set chunk callback.
        ///
        /// Called once per chunk received, before token callbacks are invoked.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&GetChatCompletionChunk`
        pub fn on_chunk<F>(mut self, f: F) -> Self
        where
            F: FnMut(&GetChatCompletionChunk) + Send + Sync + 'a,
        {
            self.on_chunk = Some(Box::new(f));
            self
        }

        /// Builder method to set reason token callback.
        ///
        /// The callback receives `(&OutputContext, token: &str)` parameters.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(&OutputContext, token: &str)`
        pub fn on_reason_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext, &str) + Send + Sync + 'a,
        {
            self.on_reason_token = Some(Box::new(f));
            self
        }

        /// Builder method to set reasoning completion callback.
        ///
        /// Called once when the reasoning phase completes for an output.
        /// This callback is invoked only once per output when reasoning transitions to Complete.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&OutputContext` and is called when reasoning completes
        pub fn on_reasoning_complete<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext) + Send + Sync + 'a,
        {
            self.on_reasoning_complete = Some(Box::new(f));
            self
        }

        /// Builder method to set content token callback.
        ///
        /// The callback receives `(&OutputContext, token: &str)` parameters.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(&OutputContext, token: &str)`
        pub fn on_content_token<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext, &str) + Send + Sync + 'a,
        {
            self.on_content_token = Some(Box::new(f));
            self
        }

        /// Builder method to set content completion callback.
        ///
        /// Called once when the content phase completes for an output.
        /// This callback is invoked only once per output when content transitions to Complete.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&OutputContext` and is called when content completes
        pub fn on_content_complete<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext) + Send + Sync + 'a,
        {
            self.on_content_complete = Some(Box::new(f));
            self
        }

        /// Builder method to set inline citations callback.
        ///
        /// Called when inline citations are present in a delta for an output.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(&OutputContext, &[InlineCitation])` and is called when citations are present
        pub fn on_inline_citations<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext, &[crate::xai_api::InlineCitation]) + Send + Sync + 'a,
        {
            self.on_inline_citations = Some(Box::new(f));
            self
        }

        /// Builder method to set tool calls callback.
        ///
        /// Called when tool calls are present in a delta, typically in the last delta per output.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `(&OutputContext, &[ToolCall])` and is called when tool calls are present
        pub fn on_tool_calls<F>(mut self, f: F) -> Self
        where
            F: FnMut(&OutputContext, &[crate::xai_api::ToolCall]) + Send + Sync + 'a,
        {
            self.on_tool_calls = Some(Box::new(f));
            self
        }

        /// Builder method to set usage callback.
        ///
        /// Called once on the last chunk with final usage statistics.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&SamplingUsage` and is called on the last chunk
        pub fn on_usage<F>(mut self, f: F) -> Self
        where
            F: FnMut(&crate::xai_api::SamplingUsage) + Send + Sync + 'a,
        {
            self.on_usage = Some(Box::new(f));
            self
        }

        /// Builder method to set citations callback.
        ///
        /// Called once on the last chunk with all citations.
        ///
        /// # Arguments
        /// * `f` - Closure that receives `&[String]` and is called on the last chunk
        pub fn on_citations<F>(mut self, f: F) -> Self
        where
            F: FnMut(&[String]) + Send + Sync + 'a,
        {
            self.on_citations = Some(Box::new(f));
            self
        }
    }

    impl<'a> Default for Consumer<'a> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Represents the status of a phase (reasoning or content) in a streaming response.
    ///
    /// This enum is used in [`OutputContext`] to track the progress of reasoning and content
    /// generation phases for each output in a multi-output stream.
    #[derive(Clone, Debug, PartialEq)]
    pub enum PhaseStatus {
        /// Initial state - the phase has not started yet.
        Init,
        /// The phase is currently in progress.
        Pending,
        /// The phase has completed.
        Complete,
    }

    /// Contextual information about an output in a streaming chat completion response.
    ///
    /// `OutputContext` provides metadata about the current output being processed, including
    /// which output it belongs to, how many total outputs are in the stream, and completion
    /// status flags for reasoning and content phases.
    ///
    /// This struct is passed to token callbacks (`on_content_token` and `on_reason_token`)
    /// and completion callbacks (`on_content_complete` and `on_reasoning_complete`) to provide
    /// context about the output's position in the stream and the state of the response generation.
    ///
    #[derive(Clone, Debug)]
    pub struct OutputContext {
        /// The total number of outputs in this streaming response.
        ///
        /// When `n > 1` is specified in the request, multiple outputs are generated
        /// concurrently. This field indicates how many outputs are being streamed.
        pub total_outputs: usize,

        /// The index of the output this token belongs to.
        ///
        /// Outputs are indexed starting from 0. Use this to distinguish tokens from
        /// different outputs when processing multi-output streams.
        pub output_index: usize,

        /// The current status of the reasoning phase for this output.
        ///
        /// - `PhaseStatus::Init`: Initial state, no reasoning content yet
        /// - `PhaseStatus::Pending`: Reasoning content is being generated
        /// - `PhaseStatus::Complete`: Reasoning phase is complete (no more reasoning tokens,
        ///   and either content has started or the output has finished)
        ///
        /// Use this to detect when the model has finished its reasoning phase
        /// and moved on to generating the final answer.
        pub reasoning_status: PhaseStatus,

        /// The current status of the content phase for this output.
        ///
        /// - `PhaseStatus::Init`: Initial state, no content yet (reasoning may still be active)
        /// - `PhaseStatus::Pending`: Content tokens are being generated
        /// - `PhaseStatus::Complete`: Content phase is complete (output has finished,
        ///   i.e., `finish_reason != 0`)
        ///
        /// Use this to detect when an output has completed and perform any
        /// final processing or cleanup.
        pub content_status: PhaseStatus,
    }

    impl OutputContext {
        /// Creates a new `OutputContext` with the specified values.
        ///
        /// # Arguments
        ///
        /// * `total_outputs` - The total number of outputs in the stream
        /// * `output_index` - The index of the output this context belongs to
        /// * `reasoning_status` - The current status of the reasoning phase
        /// * `content_status` - The current status of the content phase
        ///
        /// # Returns
        ///
        /// A new `OutputContext` instance with the provided values.
        pub fn new(
            total_outputs: usize,
            output_index: usize,
            reasoning_status: PhaseStatus,
            content_status: PhaseStatus,
        ) -> Self {
            Self {
                total_outputs,
                output_index,
                reasoning_status,
                content_status,
            }
        }
    }
}
