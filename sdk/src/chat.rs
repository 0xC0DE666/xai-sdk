//! Chat completion service client.
//!
//! Provides high-performance gRPC clients for xAI's chat completion API, supporting
//! both blocking and streaming responses with comprehensive utilities for real-time
//! token processing and response assembly.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::chat_client::ChatClient as XChatClient;

    pub type ChatClient = XChatClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `ChatClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection to xAI's chat service with automatic
    /// Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ChatClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<ChatClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XChatClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `ChatClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `ChatClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> ChatClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XChatClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ChatClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<ChatClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<ChatClient, Error> {
        let channel = common::channel::new().await?;
        let client = XChatClient::with_interceptor(channel, ClientInterceptor::new(interceptor));

        Ok(client)
    }

    /// Creates a new `ChatClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `ChatClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> ChatClient {
        XChatClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}

/// Streaming utilities for chat completions.
///
/// Provides high-performance utilities for processing real-time chat completion streams,
/// including flexible callback-based consumers and chunk assembly into complete responses.
pub mod stream {
    use crate::export::{Status, Streaming};
    use crate::xai_api::{
        CompletionMessage, CompletionOutput, FinishReason, GetChatCompletionChunk,
        GetChatCompletionResponse, InlineCitation, LogProbs, SamplingUsage, ToolCall, ToolCallType,
    };
    use std::collections::HashMap;
    use std::future::Future;
    use std::io::Write;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    /// Processes a streaming chat completion response with custom callbacks.
    ///
    /// Iterates through streaming chunks, invoking consumer callbacks for each token,
    /// completion event, and metadata. Supports multi-output streams with proper
    /// context tracking.
    ///
    /// # Arguments
    /// * `stream` - gRPC streaming response from `get_completion_chunk`
    /// * `consumer` - Configured callback consumer for handling stream events
    ///
    /// # Returns
    /// * `Ok(Vec<GetChatCompletionChunk>)` - All chunks collected from the stream
    /// * `Err(Status)` - gRPC error if streaming failed
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
                        on_chunk(&chunk).await;
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
                                on_reason_token(&output_ctx, &delta.reasoning_content).await;
                            }

                            if let Some(ref mut on_reasoning_complete) =
                                consumer.on_reasoning_complete
                            {
                                let was_complete = reasoning_complete_flags
                                    .get(&output.index)
                                    .copied()
                                    .unwrap_or(false);
                                if !was_complete && reasoning_status == PhaseStatus::Complete {
                                    on_reasoning_complete(&output_ctx).await;
                                    reasoning_complete_flags.insert(output.index, true);
                                }
                            }

                            // Content
                            if let Some(ref mut on_content_token) = consumer.on_content_token
                                && !delta.content.is_empty()
                            {
                                on_content_token(&output_ctx, &delta.content).await;
                            }

                            if let Some(ref mut on_content_complete) = consumer.on_content_complete
                            {
                                let was_complete = content_complete_flags
                                    .get(&output.index)
                                    .copied()
                                    .unwrap_or(false);
                                if !was_complete && content_status == PhaseStatus::Complete {
                                    on_content_complete(&output_ctx).await;
                                    content_complete_flags.insert(output.index, true);
                                }
                            }

                            // Inline citations
                            if let Some(ref mut on_inline_citations) = consumer.on_inline_citations
                                && !delta.citations.is_empty()
                            {
                                on_inline_citations(&output_ctx, &delta.citations).await;
                            }

                            // Tool calls - filter by type and call appropriate callbacks
                            if !delta.tool_calls.is_empty() {
                                // Separate client-side and server-side tool calls
                                let mut client_tool_calls = Vec::new();
                                let mut server_tool_calls = Vec::new();

                                for tool_call in &delta.tool_calls {
                                    if tool_call.r#type == ToolCallType::ClientSideTool.into() {
                                        client_tool_calls.push(tool_call.clone());
                                    } else {
                                        server_tool_calls.push(tool_call.clone());
                                    }
                                }

                                // Call client-side tool calls callback
                                if let Some(ref mut on_client_tool_calls) =
                                    consumer.on_client_tool_calls
                                    && !client_tool_calls.is_empty()
                                {
                                    on_client_tool_calls(&output_ctx, &client_tool_calls).await;
                                }

                                // Call server-side tool calls callback
                                if let Some(ref mut on_server_tool_calls) =
                                    consumer.on_server_tool_calls
                                    && !server_tool_calls.is_empty()
                                {
                                    on_server_tool_calls(&output_ctx, &server_tool_calls).await;
                                }
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
                    on_usage(usage).await;
                }
            }

            if let Some(ref mut on_citations) = consumer.on_citations {
                if !last_chunk.citations.is_empty() {
                    on_citations(&last_chunk.citations).await;
                }
            }
        }

        Ok(chunks)
    }

    /// Determines the reasoning phase status based on delta content and finish reason.
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

    /// Determines the content phase status based on delta content and finish reason.
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

    /// Assembles streaming chunks into a complete chat completion response.
    ///
    /// Reconstructs a full `GetChatCompletionResponse` from collected chunks by:
    /// - Grouping chunks by output index for multi-output handling
    /// - Accumulating content, reasoning, and tool calls across deltas
    /// - Preserving metadata from first chunk and usage stats from last chunk
    /// - Maintaining output ordering
    ///
    /// # Arguments
    /// * `chunks` - Vector of chunks from a streaming response
    ///
    /// # Returns
    /// * `Some(GetChatCompletionResponse)` - Complete assembled response
    /// * `None` - If chunks vector is empty
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

    /// Accumulates output data during chunk assembly process.
    #[derive(Default)]
    struct OutputData {
        content: String,
        reasoning_content: String,
        role: i32,
        tool_calls: Vec<ToolCall>,
        encrypted_content: String,
        citations: Vec<InlineCitation>,
        finish_reason: i32,
        logprobs: Option<LogProbs>,
    }

    /// Boxed future type for async callbacks. Allows references without `Send` requirement.
    pub type BoxFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    /// Callback-based consumer for processing streaming chat completion responses.
    ///
    /// Defines optional async callbacks invoked as streaming chunks arrive.
    /// Use builder methods to configure callbacks or pre-configured constructors
    /// for common use cases.
    ///
    /// # Lifetime
    ///
    /// Lifetime `'a` is inferred from callback captures. Scoped to local context
    /// if callbacks capture variables; `'static` if callbacks own all data.
    ///
    /// # Callback Signatures (execution order)
    /// All callbacks are async with `Future<Output = ()>`. Use references to avoid cloning.
    /// - `on_chunk`: `&GetChatCompletionChunk`
    /// - `on_reason_token`: `(&OutputContext, &str)`
    /// - `on_reasoning_complete`: `&OutputContext`
    /// - `on_content_token`: `(&OutputContext, &str)`
    /// - `on_content_complete`: `&OutputContext`
    /// - `on_inline_citations`: `(&OutputContext, &[InlineCitation])`
    /// - `on_client_tool_calls`: `(&OutputContext, &[ToolCall])`
    /// - `on_server_tool_calls`: `(&OutputContext, &[ToolCall])`
    /// - `on_usage`: `&SamplingUsage`
    /// - `on_citations`: `&[String]`
    pub struct Consumer<'a> {
        /// Callback invoked once per complete chunk received.
        ///
        /// Receives `&GetChatCompletionChunk`
        pub on_chunk:
            Option<Box<dyn FnMut(&GetChatCompletionChunk) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked for each reasoning token in the stream.
        ///
        /// Receives `(&OutputContext, token: &str)` — no cloning.
        pub on_reason_token:
            Option<Box<dyn FnMut(&OutputContext, &str) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked once when the reasoning phase completes for an output.
        ///
        /// This callback is called only once per output when the reasoning phase transitions
        /// to `Complete`. Useful for performing cleanup or formatting when reasoning finishes.
        ///
        /// Receives `&OutputContext`.
        pub on_reasoning_complete:
            Option<Box<dyn FnMut(&OutputContext) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked for each content token in the stream.
        ///
        /// Receives `(&OutputContext, token: &str)` — no cloning.
        pub on_content_token:
            Option<Box<dyn FnMut(&OutputContext, &str) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked once when the content phase completes for an output.
        ///
        /// This callback is called only once per output when the content phase transitions
        /// to `Complete`. Useful for performing cleanup or formatting when content generation finishes.
        ///
        /// Receives `&OutputContext`.
        pub on_content_complete:
            Option<Box<dyn FnMut(&OutputContext) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked when inline citations are present in a delta.
        ///
        /// This callback is called whenever a delta contains inline citations for an output.
        ///
        /// Receives `(&OutputContext, &[InlineCitation])`.
        pub on_inline_citations: Option<
            Box<dyn FnMut(&OutputContext, &[InlineCitation]) -> BoxFuture<'a> + Send + Sync + 'a>,
        >,

        /// Callback invoked when client-side tool calls are present.
        ///
        /// This callback is called when client-side tool calls (functions that need to be executed
        /// by the client) appear in a delta.
        ///
        /// Receives `(&OutputContext, &[ToolCall])`.
        pub on_client_tool_calls:
            Option<Box<dyn FnMut(&OutputContext, &[ToolCall]) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked when server-side tool calls are present.
        ///
        /// This callback is called when server-side tool calls (XSearch, WebSearch, CodeExecution, etc.)
        /// appear in a delta.
        ///
        /// Receives `(&OutputContext, &[ToolCall])`.
        pub on_server_tool_calls:
            Option<Box<dyn FnMut(&OutputContext, &[ToolCall]) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked once on the last chunk with usage statistics.
        ///
        /// This callback is called once when the stream completes, providing final usage statistics.
        ///
        /// Receives `&SamplingUsage` with token usage information.
        pub on_usage: Option<Box<dyn FnMut(&SamplingUsage) -> BoxFuture<'a> + Send + Sync + 'a>>,

        /// Callback invoked once on the last chunk with citations.
        ///
        /// This callback is called once when the stream completes, providing all citations
        /// from the final chunk.
        ///
        /// Receives `&[String]` with all citation URLs from the last chunk.
        pub on_citations: Option<Box<dyn FnMut(&[String]) -> BoxFuture<'a> + Send + Sync + 'a>>,
    }

    impl<'a> Consumer<'a> {
        /// Creates an empty `Consumer` with no callbacks configured.
        ///
        /// Lifetime `'a` inferred from added callbacks. Scoped to local context
        /// if callbacks capture variables.
        pub fn new() -> Self {
            Self {
                on_chunk: None,
                on_reason_token: None,
                on_reasoning_complete: None,
                on_content_token: None,
                on_content_complete: None,
                on_inline_citations: None,
                on_client_tool_calls: None,
                on_server_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Creates a `Consumer` that prints tokens to stdout in real-time.
        ///
        /// Prints reasoning and content tokens as they arrive. Only handles the first output
        /// (index 0) to avoid interleaved output. For multi-output streams, use
        /// [`with_buffered_stdout()`] instead.
        ///
        /// Returns `'static` lifetime consumer that can be extended with additional callbacks.
        pub fn with_stdout() -> Self {
            Self {
                on_chunk: None,
                on_reason_token: Some(Box::new(move |_ctx: &OutputContext, token: &str| {
                    let token = token.to_string();
                    Box::pin(async move {
                        print!("{token}");
                        std::io::stdout().flush().expect("Error flushing stdout");
                    })
                })),
                on_reasoning_complete: Some(Box::new(move |_ctx: &OutputContext| {
                    Box::pin(async move {
                        println!("\n");
                    })
                })),
                on_content_token: Some(Box::new(move |_ctx: &OutputContext, token: &str| {
                    let token = token.to_string();
                    Box::pin(async move {
                        print!("{token}");
                        std::io::stdout().flush().expect("Error flushing stdout");
                    })
                })),
                on_content_complete: Some(Box::new(move |_ctx: &OutputContext| {
                    Box::pin(async move {
                        println!("\n");
                    })
                })),
                on_inline_citations: None,
                on_client_tool_calls: None,
                on_server_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Creates a `Consumer` that buffers and prints multi-output streams cleanly.
        ///
        /// Buffers tokens per output until completion, then prints each output in
        /// labeled blocks. Prevents output interleaving in multi-output streams.
        ///
        /// Returns `'static` lifetime consumer that can be extended with additional callbacks.
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

            let buffers_reason_clone = buffers_reason.clone();
            let buffers_content_clone = buffers_content.clone();

            Self {
                on_chunk: Some(Box::new(move |chunk: &GetChatCompletionChunk| {
                    let buffers_chunk = buffers_chunk.clone();
                    let finished_clone = finished_clone.clone();
                    let outputs = chunk.outputs.clone();
                    Box::pin(async move {
                        let mut buffers = buffers_chunk.lock().unwrap();
                        let mut finished = finished_clone.lock().unwrap();

                        for output in &outputs {
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
                    })
                })),
                on_reason_token: Some(Box::new(move |ctx: &OutputContext, token: &str| {
                    let output_index = ctx.output_index as i32;
                    let token = token.to_string();
                    let buffers_reason = buffers_reason_clone.clone();
                    Box::pin(async move {
                        let mut buffers = buffers_reason.lock().unwrap();
                        let output_buf = buffers.entry(output_index).or_default();
                        output_buf.reasoning.push_str(&token);
                    })
                })),
                on_reasoning_complete: None,
                on_content_token: Some(Box::new(move |ctx: &OutputContext, token: &str| {
                    let output_index = ctx.output_index as i32;
                    let token = token.to_string();
                    let buffers_content = buffers_content_clone.clone();
                    Box::pin(async move {
                        let mut buffers = buffers_content.lock().unwrap();
                        let output_buf = buffers.entry(output_index).or_default();
                        output_buf.content.push_str(&token);
                    })
                })),
                on_content_complete: None,
                on_inline_citations: None,
                on_client_tool_calls: None,
                on_server_tool_calls: None,
                on_usage: None,
                on_citations: None,
            }
        }

        /// Sets the chunk callback, invoked once per received chunk before token callbacks.
        pub fn on_chunk<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&GetChatCompletionChunk) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_chunk = Some(Box::new(move |chunk| Box::pin(f(chunk))));
            self
        }

        /// Sets the reasoning token callback, invoked for each reasoning token.
        pub fn on_reason_token<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext, &str) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_reason_token = Some(Box::new(move |ctx, token| Box::pin(f(ctx, token))));
            self
        }

        /// Sets the reasoning completion callback, invoked once when reasoning phase ends.
        pub fn on_reasoning_complete<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_reasoning_complete = Some(Box::new(move |ctx| Box::pin(f(ctx))));
            self
        }

        /// Sets the content token callback, invoked for each content token.
        pub fn on_content_token<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext, &str) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_content_token = Some(Box::new(move |ctx, token| Box::pin(f(ctx, token))));
            self
        }

        /// Sets the content completion callback, invoked once when content generation ends.
        pub fn on_content_complete<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_content_complete = Some(Box::new(move |ctx| Box::pin(f(ctx))));
            self
        }

        /// Sets the inline citations callback, invoked when citations appear in deltas.
        pub fn on_inline_citations<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext, &[InlineCitation]) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_inline_citations =
                Some(Box::new(move |ctx, citations| Box::pin(f(ctx, citations))));
            self
        }

        /// Sets the client-side tool calls callback, invoked for client-executable tool calls.
        pub fn on_client_tool_calls<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext, &[ToolCall]) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_client_tool_calls = Some(Box::new(move |ctx, calls| Box::pin(f(ctx, calls))));
            self
        }

        /// Sets the server-side tool calls callback, invoked for server-executed tool calls.
        pub fn on_server_tool_calls<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&OutputContext, &[ToolCall]) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_server_tool_calls = Some(Box::new(move |ctx, calls| Box::pin(f(ctx, calls))));
            self
        }

        /// Sets the usage callback, invoked once with final token usage statistics.
        pub fn on_usage<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&SamplingUsage) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_usage = Some(Box::new(move |usage| Box::pin(f(usage))));
            self
        }

        /// Sets the citations callback, invoked once with all citation URLs.
        pub fn on_citations<F, Fut>(mut self, mut f: F) -> Self
        where
            F: FnMut(&[String]) -> Fut + Send + Sync + 'a,
            Fut: Future<Output = ()> + Send + Sync + 'a,
        {
            self.on_citations = Some(Box::new(move |citations| Box::pin(f(citations))));
            self
        }
    }

    impl<'a> Default for Consumer<'a> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Status of reasoning or content generation phases in streaming responses.
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
    /// Provides metadata about output position, total outputs, and generation phase status.
    /// Passed to token and completion callbacks for context-aware processing.
    ///
    #[derive(Clone, Debug)]
    pub struct OutputContext {
        /// Total number of outputs in this streaming response (when `n > 1`).
        pub total_outputs: usize,

        /// Index of this output (0-based), for distinguishing outputs in multi-output streams.
        pub output_index: usize,

        /// Current status of the reasoning phase for this output.
        pub reasoning_status: PhaseStatus,

        /// Current status of the content phase for this output.
        pub content_status: PhaseStatus,
    }

    impl OutputContext {
        /// Creates a new `OutputContext` with the specified values.
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

/// General utilities for chat related functionality.
///
/// Provides utilities for processing real-time chat completion streams,
pub mod utils {
    use crate::xai_api::{CompletionOutput, Content, Message, content};

    /// Converts a slice of `CompletionOutput` to a vector of `Message`.
    ///
    /// Maps each `CompletionOutput` from a chat completion response to a `Message`
    /// structure suitable for use in subsequent API calls. The conversion extracts
    /// the `CompletionMessage` from each output and populates all common fields
    /// between the two structures.
    ///
    /// # Arguments
    /// * `completion_outputs` - Slice of completion outputs to convert
    ///
    /// # Returns
    /// * `Vec<Message>` - Vector of messages with populated common fields
    pub fn to_messages(completion_outputs: &[CompletionOutput]) -> Vec<Message> {
        let mut messages = Vec::with_capacity(completion_outputs.len());
        for output in completion_outputs {
            if let Some(comp_msg) = &output.message {
                messages.push(Message {
                    content: vec![Content {
                        content: Some(content::Content::Text(comp_msg.content.clone())),
                    }],
                    reasoning_content: Some(comp_msg.reasoning_content.clone()),
                    role: comp_msg.role,
                    name: String::new(),
                    tool_calls: comp_msg.tool_calls.clone(),
                    encrypted_content: comp_msg.encrypted_content.clone(),
                    tool_call_id: None,
                });
            }
        }
        messages
    }
}
