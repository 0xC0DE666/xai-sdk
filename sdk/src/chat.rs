//! Chat completion service client.
//!
//! Provides high-performance gRPC clients for xAI's chat completion API, supporting
//! both blocking and streaming responses with comprehensive utilities for real-time
//! token processing and response assembly.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::tonic::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::tonic::transport::{Channel, Error};
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
/// Provides utilities for processing real-time chat completion streams via [`stream::Event`],
/// and for assembling chunks into complete responses.
pub mod stream {
    use crate::export::tonic::Status;
    use crate::xai_api::{
        CompletionMessage, CompletionOutput, FinishReason, GetChatCompletionChunk,
        GetChatCompletionResponse, InlineCitation, LogProbs, SamplingUsage, ToolCall, ToolCallType,
    };
    use crate::export::futures::sink::{drain, Sink};
    use crate::export::futures::stream::Stream;
    use crate::export::futures::{SinkExt, StreamExt};
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::io::Write;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    #[derive(Debug, Clone)]
    struct OutputStats {
        index: i32,
        total_reasoning_tokens: usize,
        total_content_tokens: usize,
        finish_reason: FinishReason,
    }

    impl OutputStats {
        fn init(index: i32, finish_reason: FinishReason) -> Self {
            Self {
                index,
                total_reasoning_tokens: 0,
                total_content_tokens: 0,
                finish_reason,
            }
        }

        fn inc(&mut self, reason_token: &str, content_token: &str) {
            if !reason_token.is_empty() {
                self.total_reasoning_tokens += 1;
            }
            if !content_token.is_empty() {
                self.total_content_tokens += 1;
            }
        }

        fn merge(&mut self, other: &Self) {
            if self.index == other.index {
                self.total_reasoning_tokens += other.total_reasoning_tokens;
                self.total_content_tokens += other.total_content_tokens;
                self.finish_reason = other.finish_reason
            }
        }
    }

    /// Status of reasoning or content generation phases in streaming responses.
    #[derive(Clone, Debug, PartialEq)]
    pub enum PhaseStatus {
        Init,
        Start,
        Pending,
        Complete,
    }

    /// Contextual information about an output in a streaming chat completion response.
    #[derive(Clone, Debug)]
    pub struct OutputContext {
        pub total_outputs: usize,
        pub output_index: usize,
        pub reasoning_status: PhaseStatus,
        pub content_status: PhaseStatus,
    }

    impl OutputContext {
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

    /// A single event from a streaming chat completion, as produced by [`process`].
    #[derive(Debug)]
    pub enum Event {
        Chunk(GetChatCompletionChunk),
        ReasoningStart(OutputContext),
        ReasoningToken(OutputContext, String),
        ReasoningComplete(OutputContext),
        ContentStart(OutputContext),
        ContentToken(OutputContext, String),
        ContentComplete(OutputContext),
        InlineCitations(OutputContext, Vec<InlineCitation>),
        ClientToolCalls(OutputContext, Vec<ToolCall>),
        ServerToolCalls(OutputContext, Vec<ToolCall>),
        Citations(Vec<String>),
        Usage(Option<SamplingUsage>),
        Complete,
        Error(Status),
    }

    async fn emit<Si>(events: &mut Si, event: Event)
    where
        Si: Sink<Event> + Unpin,
    {
        let _ = events.send(event).await;
    }

    /// Processes a streaming chat completion response, emitting [`Event`]s.
    ///
    /// Iterates through streaming chunks, sending events for each token,
    /// completion event, and metadata. Supports multi-output streams with proper
    /// context tracking.
    ///
    /// # Arguments
    /// * `stream` - Any stream yielding `Result<GetChatCompletionChunk, Status>` (e.g. from
    ///   `get_completion_chunk` or a mock). Must implement `Stream + Send + Unpin + 'static`.
    /// * `events` - Sink that receives [`Event`]s (e.g. [`export::futures::channel::mpsc::unbounded`] sender)
    ///
    /// # Returns
    /// * `Ok(Vec<GetChatCompletionChunk>)` - All chunks collected from the stream
    /// * `Err(Status)` - gRPC error if streaming failed (after emitting [`Event::Error`])
    pub async fn process<S, Si>(
        mut stream: S,
        mut events: Si,
    ) -> Result<Vec<GetChatCompletionChunk>, Status>
    where
        S: Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static,
        Si: Sink<Event> + Send + Unpin,
    {
        let mut chunks: Vec<GetChatCompletionChunk> = Vec::new();
        let mut output_stats: HashMap<i32, OutputStats> = HashMap::new();
        let mut reasoning_start_fired: HashMap<i32, bool> = HashMap::new();
        let mut reasoning_complete_fired: HashMap<i32, bool> = HashMap::new();
        let mut content_start_fired: HashMap<i32, bool> = HashMap::new();
        let mut content_complete_fired: HashMap<i32, bool> = HashMap::new();
        let mut last_chunk: Option<GetChatCompletionChunk> = None;
        let mut max_output_index_seen: i32 = -1;

        loop {
            match stream.next().await {
                None => break,
                Some(Err(status)) => {
                    emit(&mut events, Event::Error(status.clone())).await;
                    return Err(status);
                }
                Some(Ok(chunk)) => {
                    emit(&mut events, Event::Chunk(chunk.clone())).await;

                    for output in &chunk.outputs {
                        let cur_output_index = output.index;

                        // Update max early so total_outputs is as accurate as possible
                        max_output_index_seen = max_output_index_seen.max(cur_output_index);

                        // Always start with the latest finish_reason from this chunk
                        let mut cur_output_stats =
                            OutputStats::init(cur_output_index, output.finish_reason());

                        let delta = output.delta.as_ref();

                        // Increment token counts only if there is a delta
                        if let Some(delta) = delta {
                            cur_output_stats.inc(&delta.reasoning_content, &delta.content);
                        }

                        // Merge current chunk data into accumulated stats
                        // Chained to avoid the move error
                        let merged = output_stats
                            .entry(cur_output_index)
                            .and_modify(|e| e.merge(&cur_output_stats))
                            .or_insert(cur_output_stats);

                        let (reasoning_status, content_status) = get_output_status(merged);

                        let total_outputs = (max_output_index_seen + 1) as usize;

                        let output_ctx = OutputContext::new(
                            total_outputs,
                            cur_output_index as usize,
                            reasoning_status.clone(),
                            content_status.clone(),
                        );

                        // #################### Reasoning ####################
                        if let Some(delta) = delta
                            && !delta.reasoning_content.is_empty()
                        {
                            // Reasoning start (once)
                            if !reasoning_start_fired
                                .get(&cur_output_index)
                                .copied()
                                .unwrap_or(false)
                            {
                                emit(&mut events, Event::ReasoningStart(output_ctx.clone())).await;
                                reasoning_start_fired.insert(cur_output_index, true);
                            }
                            // Reasoning tokens (multiple)
                            emit(
                                &mut events,
                                Event::ReasoningToken(
                                    output_ctx.clone(),
                                    delta.reasoning_content.clone(),
                                ),
                            )
                            .await;
                        }

                        // Phase completion callback – fire immediately when we detect Complete
                        // (runs even on chunks without delta, in case finish_reason updated)
                        // Reasoning complete (once)
                        if reasoning_status == PhaseStatus::Complete
                            && !reasoning_complete_fired
                                .get(&cur_output_index)
                                .copied()
                                .unwrap_or(false)
                            && merged.total_reasoning_tokens > 0
                        {
                            emit(&mut events, Event::ReasoningComplete(output_ctx.clone())).await;
                            reasoning_complete_fired.insert(cur_output_index, true);
                        }

                        // #################### Content ####################
                        if let Some(delta) = delta
                            && !delta.content.is_empty()
                        {
                            // Content start (once)
                            if !content_start_fired
                                .get(&cur_output_index)
                                .copied()
                                .unwrap_or(false)
                            {
                                emit(&mut events, Event::ContentStart(output_ctx.clone())).await;
                                content_start_fired.insert(cur_output_index, true);
                            }
                            // Content tokens (multiple)
                            emit(
                                &mut events,
                                Event::ContentToken(output_ctx.clone(), delta.content.clone()),
                            )
                            .await;
                        }

                        // Phase completion callback – fire immediately when we detect Complete
                        // (runs even on chunks without delta, in case finish_reason updated)
                        // Content complete (once)
                        if content_status == PhaseStatus::Complete
                            && !content_complete_fired
                                .get(&cur_output_index)
                                .copied()
                                .unwrap_or(false)
                            && merged.total_content_tokens > 0
                        {
                            emit(&mut events, Event::ContentComplete(output_ctx.clone())).await;
                            content_complete_fired.insert(cur_output_index, true);
                        }

                        // #################### Citations, Tool Calls ####################
                        if let Some(delta) = delta {
                            // Inline citations
                            if !delta.citations.is_empty() {
                                emit(
                                    &mut events,
                                    Event::InlineCitations(
                                        output_ctx.clone(),
                                        delta.citations.clone(),
                                    ),
                                )
                                .await;
                            }

                            // Tool calls
                            if !delta.tool_calls.is_empty() {
                                let mut client_tool_calls = Vec::new();
                                let mut server_tool_calls = Vec::new();
                                for tool_call in &delta.tool_calls {
                                    if tool_call.r#type == ToolCallType::ClientSideTool.into() {
                                        client_tool_calls.push(tool_call.clone());
                                    } else {
                                        server_tool_calls.push(tool_call.clone());
                                    }
                                }
                                if !client_tool_calls.is_empty() {
                                    emit(
                                        &mut events,
                                        Event::ClientToolCalls(
                                            output_ctx.clone(),
                                            client_tool_calls,
                                        ),
                                    )
                                    .await;
                                }
                                if !server_tool_calls.is_empty() {
                                    emit(
                                        &mut events,
                                        Event::ServerToolCalls(
                                            output_ctx.clone(),
                                            server_tool_calls,
                                        ),
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    last_chunk = Some(chunk.clone());
                    chunks.push(chunk);
                }
            }
        }

        // Final metadata events (only on last chunk)
        if let Some(ref last_chunk) = last_chunk {
            if let Some(ref usage) = last_chunk.usage {
                emit(&mut events, Event::Usage(Some(usage.clone()))).await;
            }
            if !last_chunk.citations.is_empty() {
                emit(&mut events, Event::Citations(last_chunk.citations.clone())).await;
            }
        }

        emit(&mut events, Event::Complete).await;
        Ok(chunks)
    }

    /// Collects all chunks from a stream, discarding events.
    pub async fn collect<S>(stream: S) -> Result<Vec<GetChatCompletionChunk>, Status>
    where
        S: Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static,
    {
        process(stream, drain()).await
    }

    struct StdoutSink;

    impl Sink<Event> for StdoutSink {
        type Error = Infallible;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: Event) -> Result<(), Self::Error> {
            match item {
                Event::ReasoningToken(_, ref t) => {
                    print!("{t}");
                    let _ = std::io::stdout().flush();
                }
                Event::ReasoningComplete(_) => println!(),
                Event::ContentToken(_, ref t) => {
                    print!("{t}");
                    let _ = std::io::stdout().flush();
                }
                Event::ContentComplete(_) => println!(),
                _ => {}
            }
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    /// Processes a stream and prints tokens to stdout (single-output friendly).
    pub async fn process_stdout<S>(stream: S) -> Result<Vec<GetChatCompletionChunk>, Status>
    where
        S: Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static,
    {
        process(stream, StdoutSink).await
    }

    #[derive(Default)]
    struct ChoiceBuffer {
        content: String,
        reasoning: String,
    }

    struct BufferedStdoutSink {
        buffers: Arc<Mutex<HashMap<i32, ChoiceBuffer>>>,
        finished: Arc<Mutex<HashMap<i32, bool>>>,
    }

    impl BufferedStdoutSink {
        fn new() -> Self {
            Self {
                buffers: Arc::new(Mutex::new(HashMap::new())),
                finished: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn on_chunk(&self, chunk: &GetChatCompletionChunk) {
            let mut buffers = self.buffers.lock().unwrap();
            let mut finished = self.finished.lock().unwrap();
            for output in &chunk.outputs {
                let idx = output.index;
                if output.finish_reason != 0 && !finished.contains_key(&idx) {
                    finished.insert(idx, true);
                    if let Some(output_buf) = buffers.remove(&idx) {
                        println!("\n--- Output {idx} ---");
                        if !output_buf.reasoning.is_empty() {
                            println!("Reasoning:\n{}\n", output_buf.reasoning);
                        }
                        if !output_buf.content.is_empty() {
                            println!("Content:\n{}\n", output_buf.content);
                        }
                        println!("Finish reason: {}\n", output.finish_reason);
                        let _ = std::io::stdout().flush();
                    }
                }
            }
        }
    }

    impl Sink<Event> for BufferedStdoutSink {
        type Error = Infallible;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: Event) -> Result<(), Self::Error> {
            match item {
                Event::Chunk(ref chunk) => self.on_chunk(chunk),
                Event::ReasoningToken(ref ctx, ref t) => {
                    let idx = ctx.output_index as i32;
                    self.buffers
                        .lock()
                        .unwrap()
                        .entry(idx)
                        .or_default()
                        .reasoning
                        .push_str(t);
                }
                Event::ContentToken(ref ctx, ref t) => {
                    let idx = ctx.output_index as i32;
                    self.buffers
                        .lock()
                        .unwrap()
                        .entry(idx)
                        .or_default()
                        .content
                        .push_str(t);
                }
                _ => {}
            }
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    /// Processes a stream and prints each output in labeled blocks (multi-output friendly).
    pub async fn process_buffered_stdout<S>(
        stream: S,
    ) -> Result<Vec<GetChatCompletionChunk>, Status>
    where
        S: Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static,
    {
        process(stream, BufferedStdoutSink::new()).await
    }

    /// Returns (reasoning_status, content_status) for the current output from accumulated stats.
    fn get_output_status(cur_output_stats: &OutputStats) -> (PhaseStatus, PhaseStatus) {
        let is_finished = cur_output_stats.finish_reason != FinishReason::ReasonInvalid;
        let r = cur_output_stats.total_reasoning_tokens;
        let c = cur_output_stats.total_content_tokens;

        let reasoning_status = if is_finished {
            PhaseStatus::Complete
        } else if r == 0 && c > 0 {
            PhaseStatus::Complete // no-reasoning model, content started
        } else if r > 0 && c > 0 {
            PhaseStatus::Complete // reasoning done, content in progress
        } else if r == 1 && c == 0 {
            PhaseStatus::Start // first reasoning token
        } else if r > 1 && c == 0 {
            PhaseStatus::Pending
        } else {
            PhaseStatus::Init // r == 0 && c == 0
        };

        let content_status = if is_finished {
            PhaseStatus::Complete
        } else if c == 1 {
            PhaseStatus::Start // first content token
        } else if c > 1 {
            PhaseStatus::Pending
        } else {
            PhaseStatus::Init
        };

        (reasoning_status, content_status)
    }

    /// Appends whole tool calls from a delta. xAI sends each tool call in full (not argument fragments).
    /// If the same [`ToolCall::id`] appears again, the later call replaces the earlier one.
    fn append_tool_calls(acc: &mut Vec<ToolCall>, incoming: &[ToolCall]) {
        for call in incoming {
            if !call.id.is_empty()
                && let Some(i) = acc.iter().position(|t| t.id == call.id)
            {
                acc[i] = call.clone();
            } else {
                acc.push(call.clone());
            }
        }
    }

    /// Merges inline citations by [`InlineCitation::id`]; later fragments update the same citation.
    fn merge_inline_citations(acc: &mut Vec<InlineCitation>, incoming: &[InlineCitation]) {
        for cite in incoming {
            if !cite.id.is_empty()
                && let Some(i) = acc.iter().position(|c| c.id == cite.id)
            {
                let existing = &mut acc[i];
                if cite.start_index != 0 {
                    existing.start_index = cite.start_index;
                }
                if cite.end_index != 0 {
                    existing.end_index = cite.end_index;
                }
                if cite.citation.is_some() {
                    existing.citation.clone_from(&cite.citation);
                }
            } else {
                acc.push(cite.clone());
            }
        }
    }

    /// Assembles streaming chunks into a complete chat completion response.
    ///
    /// Reconstructs a full `GetChatCompletionResponse` from collected chunks by:
    /// - Grouping chunks by output index for multi-output handling
    /// - Accumulating content and reasoning across deltas
    /// - Appending whole tool calls from each delta (replacing by id if re-sent)
    /// - Merging inline citations by id
    /// - Preserving metadata from the first chunk and usage/citations/debug from the last relevant chunk
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

                    append_tool_calls(&mut output_data.tool_calls, &delta.tool_calls);
                    merge_inline_citations(&mut output_data.citations, &delta.citations);
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

        let debug_output = chunks.iter().rev().find_map(|c| c.debug_output.clone());

        Some(GetChatCompletionResponse {
            id: first_chunk.id.clone(),
            outputs,
            created: first_chunk.created.clone(),
            model: first_chunk.model.clone(),
            system_fingerprint: first_chunk.system_fingerprint.clone(),
            usage,
            citations,
            settings: None, // Not present on streaming chunks
            debug_output,
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

    #[cfg(test)]
    mod tests {
        use super::*;

        fn stats(
            index: i32,
            reasoning: usize,
            content: usize,
            finish_reason: FinishReason,
        ) -> OutputStats {
            OutputStats {
                index,
                total_reasoning_tokens: reasoning,
                total_content_tokens: content,
                finish_reason,
            }
        }

        #[test]
        fn get_output_status_init_init() {
            let s = stats(0, 0, 0, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Init);
            assert_eq!(c, PhaseStatus::Init);
        }

        #[test]
        fn get_output_status_pending_init() {
            // Reasoning in progress (more than one token), no content yet
            let s = stats(0, 2, 0, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Pending);
            assert_eq!(c, PhaseStatus::Init);
        }

        #[test]
        fn get_output_status_complete_pending() {
            // No reasoning, content in progress (more than one token)
            let s = stats(0, 0, 2, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Pending);
        }

        #[test]
        fn get_output_status_complete_complete() {
            let s = stats(0, 0, 0, FinishReason::ReasonStop);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Complete);
        }

        #[test]
        fn get_output_status_complete_complete_with_tokens() {
            let s = stats(0, 5, 10, FinishReason::ReasonStop);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Complete);
        }

        #[test]
        fn get_output_status_tool_calls_only() {
            let s = stats(0, 0, 0, FinishReason::ReasonToolCalls);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Complete);
        }

        #[test]
        fn get_output_status_reasoning_done_content_pending() {
            // Reasoning phase finished, content streaming (not yet finished)
            let s = stats(0, 5, 10, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Pending);
        }

        #[test]
        fn get_output_status_reasoning_start() {
            // First reasoning token, no content yet
            let s = stats(0, 1, 0, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Start);
            assert_eq!(c, PhaseStatus::Init);
        }

        #[test]
        fn get_output_status_content_start() {
            // First content token (no reasoning), stream not finished
            let s = stats(0, 0, 1, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete); // no-reasoning model
            assert_eq!(c, PhaseStatus::Start);
        }

        #[test]
        fn get_output_status_content_start_with_reasoning() {
            // First content token after reasoning (r=1, c=1 in same or previous chunk)
            let s = stats(0, 1, 1, FinishReason::ReasonInvalid);
            let (r, c) = get_output_status(&s);
            assert_eq!(r, PhaseStatus::Complete);
            assert_eq!(c, PhaseStatus::Start);
        }
    }
}

pub mod traits {
    use crate::xai_api::{Content, Message};

    pub trait ToContent {
        fn to_content(&self) -> Content;
    }

    pub trait ToMessage {
        fn to_message(&self) -> Message;
    }

    pub trait ToMessages {
        fn to_messages(&self) -> Vec<Message>;
    }
}

/// General utilities for chat related functionality.
///
/// Provides utilities for converting completion outputs to messages and related chat operations.
pub mod utils {
    use super::traits::ToContent;
    use crate::xai_api::{CompletionOutput, Content, Message, MessageRole, content};

    impl From<&str> for Content {
        fn from(value: &str) -> Self {
            Content {
                content: Some(content::Content::Text(value.to_string())),
            }
        }
    }

    impl ToContent for &str {
        fn to_content(&self) -> Content {
            (*self).into()
        }
    }

    impl ToContent for String {
        fn to_content(&self) -> Content {
            self.as_str().into()
        }
    }

    /// Creates a chat message from a role and generic content-convertible parts.
    pub fn message_from<T>(role: MessageRole, content: &[T]) -> Message
    where
        T: ToContent,
    {
        let content = content.iter().map(|c| c.to_content()).collect();

        Message {
            content,
            role: role.into(),
            ..Default::default()
        }
    }

    /// Converts a slice of `CompletionOutput` to a vector of `Message`.
    ///
    /// Maps each `CompletionOutput` from a chat completion response to a `Message`
    /// structure suitable for use in subsequent API calls. The conversion extracts
    /// the `CompletionMessage` from each output and populates all common fields
    /// between the two structures.
    ///
    /// # Arguments
    /// * `completions` - Slice of completion outputs to convert
    ///
    /// # Returns
    /// * `Vec<Message>` - Vector of messages with populated common fields
    pub fn to_messages(completions: &[CompletionOutput]) -> Vec<Message> {
        let mut messages = Vec::with_capacity(completions.len());
        for output in completions {
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
