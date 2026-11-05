use tonic::Status;
use xai_sdk::api::tool_call;
use xai_sdk::api::{
    CompletionMessage, CompletionOutput, CompletionOutputChunk, Delta, FinishReason, FunctionCall,
    GetChatCompletionChunk, InlineCitation, MessageRole, SamplingUsage, ToolCall, ToolCallType,
    content::Content as ApiContent,
};
use xai_sdk::chat::stream::{Event, OutputContext, PhaseStatus, assemble, process};
use xai_sdk::chat::utils::to_messages;
use xai_sdk::export::futures::StreamExt;
use xai_sdk::export::futures::channel::mpsc;
use xai_sdk::export::futures::stream::{self, Stream};

/// Collect all events from the receiver after process completes.
async fn collect_events(mut rcv: mpsc::UnboundedReceiver<Event>) -> Vec<Event> {
    let mut out = Vec::new();
    while let Some(ev) = rcv.next().await {
        out.push(ev);
    }
    out
}

async fn run_process<S>(stream: S) -> Result<(Vec<GetChatCompletionChunk>, Vec<Event>), Status>
where
    S: Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static,
{
    let (tx, rx) = mpsc::unbounded();
    let chunks = process(stream, tx).await?;
    Ok((chunks, collect_events(rx).await))
}

#[test]
fn test_output_context_new() {
    let ctx = OutputContext::new(2, 1, PhaseStatus::Pending, PhaseStatus::Init);
    assert_eq!(ctx.total_outputs, 2);
    assert_eq!(ctx.output_index, 1);
    assert_eq!(ctx.reasoning_status, PhaseStatus::Pending);
    assert_eq!(ctx.content_status, PhaseStatus::Init);
}

#[test]
fn test_output_context_clone() {
    let ctx = OutputContext::new(1, 0, PhaseStatus::Complete, PhaseStatus::Complete);
    let cloned = ctx.clone();
    assert_eq!(cloned.total_outputs, ctx.total_outputs);
    assert_eq!(cloned.output_index, ctx.output_index);
    assert_eq!(cloned.reasoning_status, ctx.reasoning_status);
    assert_eq!(cloned.content_status, ctx.content_status);
}

#[test]
fn test_output_context_with_complete_phases() {
    let ctx = OutputContext::new(3, 2, PhaseStatus::Complete, PhaseStatus::Complete);
    assert_eq!(ctx.total_outputs, 3);
    assert_eq!(ctx.output_index, 2);
}

#[test]
fn test_output_context_with_complete_phases_clone() {
    let ctx = OutputContext::new(2, 1, PhaseStatus::Complete, PhaseStatus::Complete);
    let cloned = ctx.clone();
    assert_eq!(cloned.total_outputs, ctx.total_outputs);
    assert_eq!(cloned.output_index, ctx.output_index);
}

#[test]
fn test_phase_status_equality() {
    assert_eq!(PhaseStatus::Init, PhaseStatus::Init);
    assert_eq!(PhaseStatus::Pending, PhaseStatus::Pending);
    assert_eq!(PhaseStatus::Complete, PhaseStatus::Complete);
    assert_ne!(PhaseStatus::Init, PhaseStatus::Pending);
    assert_ne!(PhaseStatus::Pending, PhaseStatus::Complete);
}

#[test]
fn test_assemble_empty_chunks() {
    let chunks = vec![];
    let result = assemble(chunks);
    assert!(result.is_none());
}

#[test]
fn test_assemble_single_chunk_empty_outputs() {
    let chunk = GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![],
        created: None,
        model: "m".to_string(),
        system_fingerprint: String::new(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    };
    let result = assemble(vec![chunk]);
    assert!(result.is_some());
    let response = result.unwrap();
    assert!(response.outputs.is_empty());
    assert_eq!(response.id, "id");
}

#[test]
fn test_assemble_single_chunk_single_choice() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.content = "Hello".to_string();
    delta.reasoning_content = "Thinking...".to_string();
    output.delta = Some(delta);

    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.id, "test-id");
    assert_eq!(response.model, "test-model");
    assert_eq!(response.outputs.len(), 1);
    assert_eq!(response.outputs[0].index, 0);
    assert!(response.outputs[0].message.is_some());
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.content, "Hello");
    assert_eq!(message.reasoning_content, "Thinking...");
}

#[test]
fn test_assemble_multiple_chunks_accumulate_content() {
    // Create multiple chunks that accumulate content
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    let mut delta1 = Delta::default();
    delta1.content = "Hello".to_string();
    output1.delta = Some(delta1);
    chunk1.outputs = vec![output1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 0;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.content = " World".to_string();
    output2.delta = Some(delta2);
    chunk2.outputs = vec![output2];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.outputs.len(), 1);
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.content, "Hello World");
}

#[test]
fn test_assemble_multiple_choices() {
    // Test assembling chunks with multiple outputs
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    output1.finish_reason = FinishReason::ReasonStop.into();
    let mut delta1 = Delta::default();
    delta1.content = "Output 1".to_string();
    output1.delta = Some(delta1);

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 1;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.content = "Output 2".to_string();
    output2.delta = Some(delta2);

    chunk.outputs = vec![output1, output2];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.outputs.len(), 2);
    // Outputs should be sorted by index
    assert_eq!(response.outputs[0].index, 0);
    assert_eq!(response.outputs[1].index, 1);
    assert_eq!(
        response.outputs[0].message.as_ref().unwrap().content,
        "Output 1"
    );
    assert_eq!(
        response.outputs[1].message.as_ref().unwrap().content,
        "Output 2"
    );
}

#[test]
fn test_assemble_preserves_metadata() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "unique-id-123".to_string();
    chunk.model = "grok-3-latest".to_string();
    chunk.system_fingerprint = "fp-123".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();
    let mut delta = Delta::default();
    delta.content = "Test".to_string();
    output.delta = Some(delta);
    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.id, "unique-id-123");
    assert_eq!(response.model, "grok-3-latest");
    assert_eq!(response.system_fingerprint, "fp-123".to_string());
}

#[test]
fn test_assemble_accumulates_reasoning_content() {
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    let mut delta1 = Delta::default();
    delta1.reasoning_content = "Step 1: ".to_string();
    output1.delta = Some(delta1);
    chunk1.outputs = vec![output1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 0;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.reasoning_content = "Step 2".to_string();
    delta2.content = "Answer".to_string();
    output2.delta = Some(delta2);
    chunk2.outputs = vec![output2];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.reasoning_content, "Step 1: Step 2");
    assert_eq!(message.content, "Answer");
}

#[test]
fn test_assemble_uses_last_chunk_for_usage() {
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();
    chunk1.usage = Some(SamplingUsage {
        prompt_tokens: 10,
        completion_tokens: 0,
        total_tokens: 10,
        ..Default::default()
    });

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    let mut delta1 = Delta::default();
    delta1.content = "Hello".to_string();
    output1.delta = Some(delta1);
    chunk1.outputs = vec![output1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();
    chunk2.usage = Some(SamplingUsage {
        prompt_tokens: 10,
        completion_tokens: 5,
        total_tokens: 15,
        ..Default::default()
    });

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 0;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.content = " World".to_string();
    output2.delta = Some(delta2);
    chunk2.outputs = vec![output2];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert!(response.usage.is_some());
    let usage = response.usage.as_ref().unwrap();
    // Should use the last chunk's usage (completion_tokens: 5)
    assert_eq!(usage.completion_tokens, 5);
    assert_eq!(usage.total_tokens, 15);
}

#[test]
fn test_assemble_updates_finish_reason_from_latest_chunk() {
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    output1.finish_reason = FinishReason::ReasonInvalid.into();
    let mut delta1 = Delta::default();
    delta1.content = "Hello".to_string();
    output1.delta = Some(delta1);
    chunk1.outputs = vec![output1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 0;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.content = " World".to_string();
    output2.delta = Some(delta2);
    chunk2.outputs = vec![output2];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(
        response.outputs[0].finish_reason,
        FinishReason::ReasonStop.into()
    );
}

#[test]
fn test_assemble_handles_multiple_outputs_with_different_indices() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output0 = CompletionOutputChunk::default();
    output0.index = 0;
    output0.finish_reason = FinishReason::ReasonStop.into();
    let mut delta0 = Delta::default();
    delta0.content = "Output 0".to_string();
    output0.delta = Some(delta0);

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 2;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.content = "Output 2".to_string();
    output2.delta = Some(delta2);

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 1;
    output1.finish_reason = FinishReason::ReasonStop.into();
    let mut delta1 = Delta::default();
    delta1.content = "Output 1".to_string();
    output1.delta = Some(delta1);

    // Add outputs in non-sequential order
    chunk.outputs = vec![output0, output2, output1];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.outputs.len(), 3);
    // Should be sorted by index
    assert_eq!(response.outputs[0].index, 0);
    assert_eq!(response.outputs[1].index, 1);
    assert_eq!(response.outputs[2].index, 2);
    assert_eq!(
        response.outputs[0].message.as_ref().unwrap().content,
        "Output 0"
    );
    assert_eq!(
        response.outputs[1].message.as_ref().unwrap().content,
        "Output 1"
    );
    assert_eq!(
        response.outputs[2].message.as_ref().unwrap().content,
        "Output 2"
    );
}

#[test]
fn test_assemble_handles_outputs_across_multiple_chunks() {
    // Test that outputs with the same index across different chunks are accumulated
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();

    let mut output1_0 = CompletionOutputChunk::default();
    output1_0.index = 0;
    let mut delta1_0 = Delta::default();
    delta1_0.content = "Hello".to_string();
    output1_0.delta = Some(delta1_0);

    let mut output1_1 = CompletionOutputChunk::default();
    output1_1.index = 1;
    let mut delta1_1 = Delta::default();
    delta1_1.content = "Hi".to_string();
    output1_1.delta = Some(delta1_1);

    chunk1.outputs = vec![output1_0, output1_1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();

    let mut output2_0 = CompletionOutputChunk::default();
    output2_0.index = 0;
    output2_0.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2_0 = Delta::default();
    delta2_0.content = " World".to_string();
    output2_0.delta = Some(delta2_0);

    let mut output2_1 = CompletionOutputChunk::default();
    output2_1.index = 1;
    output2_1.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2_1 = Delta::default();
    delta2_1.content = " There".to_string();
    output2_1.delta = Some(delta2_1);

    chunk2.outputs = vec![output2_0, output2_1];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    assert_eq!(response.outputs.len(), 2);
    assert_eq!(
        response.outputs[0].message.as_ref().unwrap().content,
        "Hello World"
    );
    assert_eq!(
        response.outputs[1].message.as_ref().unwrap().content,
        "Hi There"
    );
}

#[test]
fn test_to_messages_empty_vector() {
    let completion_outputs: Vec<CompletionOutput> = vec![];
    let messages = to_messages(&completion_outputs);
    assert!(messages.is_empty());
}

#[test]
fn test_to_messages_single_completion_message() {
    let completion_message = CompletionMessage {
        content: "Hello, world!".to_string(),
        reasoning_content: "Thinking step by step...".to_string(),
        role: MessageRole::RoleAssistant.into(),
        tool_calls: vec![ToolCall {
            id: "call-123".to_string(),
            ..Default::default()
        }],
        encrypted_content: "encrypted-data".to_string(),
        citations: vec![InlineCitation {
            id: "cit-1".to_string(),
            ..Default::default()
        }],
    };

    let completion_output = CompletionOutput {
        message: Some(completion_message),
        ..Default::default()
    };

    let completion_outputs = vec![completion_output];
    let messages = to_messages(&completion_outputs);

    assert_eq!(messages.len(), 1);
    let message = &messages[0];

    // Check content conversion from String to Vec<Content>
    assert_eq!(message.content.len(), 1);
    match &message.content[0].content {
        Some(ApiContent::Text(text)) => {
            assert_eq!(text, "Hello, world!");
        }
        _ => panic!("Expected Text content"),
    }

    // Check other fields are copied correctly
    assert_eq!(
        message.reasoning_content,
        Some("Thinking step by step...".to_string())
    );
    assert_eq!(message.role, MessageRole::RoleAssistant.into());
    assert_eq!(message.tool_calls.len(), 1);
    assert_eq!(message.tool_calls[0].id, "call-123");
    assert_eq!(message.encrypted_content, "encrypted-data");

    // Check default values for Message-specific fields
    assert_eq!(message.name, String::new());
    assert!(message.tool_call_id.is_none());
}

#[test]
fn test_to_messages_multiple_completion_messages() {
    let completion_output1 = CompletionOutput {
        message: Some(CompletionMessage {
            content: "First message".to_string(),
            reasoning_content: "First reasoning".to_string(),
            role: MessageRole::RoleUser.into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let completion_output2 = CompletionOutput {
        message: Some(CompletionMessage {
            content: "Second message".to_string(),
            reasoning_content: "Second reasoning".to_string(),
            role: MessageRole::RoleAssistant.into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let completion_outputs = vec![completion_output1, completion_output2];
    let messages = to_messages(&completion_outputs);

    assert_eq!(messages.len(), 2);

    // Check first message
    assert_eq!(messages[0].role, MessageRole::RoleUser.into());
    match &messages[0].content[0].content {
        Some(ApiContent::Text(text)) => {
            assert_eq!(text, "First message");
        }
        _ => panic!("Expected Text content"),
    }

    // Check second message
    assert_eq!(messages[1].role, MessageRole::RoleAssistant.into());
    match &messages[1].content[0].content {
        Some(ApiContent::Text(text)) => {
            assert_eq!(text, "Second message");
        }
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_to_messages_preserves_tool_calls() {
    use xai_sdk::api::{FunctionCall, tool_call};

    let completion_message = CompletionMessage {
        tool_calls: vec![
            ToolCall {
                id: "tool-1".to_string(),
                r#type: 0,
                tool: Some(tool_call::Tool::Function(FunctionCall {
                    name: "search".to_string(),
                    arguments: r#"{"query": "test"}"#.to_string(),
                })),
                ..Default::default()
            },
            ToolCall {
                id: "tool-2".to_string(),
                r#type: 1,
                tool: Some(tool_call::Tool::Function(FunctionCall {
                    name: "calculate".to_string(),
                    arguments: r#"{"expr": "2+2"}"#.to_string(),
                })),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let completion_output = CompletionOutput {
        message: Some(completion_message),
        ..Default::default()
    };

    let completion_outputs = vec![completion_output];
    let messages = to_messages(&completion_outputs);

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].tool_calls.len(), 2);
    assert_eq!(messages[0].tool_calls[0].id, "tool-1");
    match &messages[0].tool_calls[0].tool {
        Some(tool_call::Tool::Function(func)) => {
            assert_eq!(func.name, "search");
        }
        _ => panic!("Expected Function tool"),
    }
    assert_eq!(messages[0].tool_calls[1].id, "tool-2");
    match &messages[0].tool_calls[1].tool {
        Some(tool_call::Tool::Function(func)) => {
            assert_eq!(func.name, "calculate");
        }
        _ => panic!("Expected Function tool"),
    }
}

#[test]
fn test_to_messages_empty_content() {
    let completion_message = CompletionMessage {
        content: String::new(),
        reasoning_content: String::new(),
        ..Default::default()
    };

    let completion_output = CompletionOutput {
        message: Some(completion_message),
        ..Default::default()
    };

    let completion_outputs = vec![completion_output];
    let messages = to_messages(&completion_outputs);

    assert_eq!(messages.len(), 1);
    // Even with empty content, we should still have one Content element
    assert_eq!(messages[0].content.len(), 1);
    match &messages[0].content[0].content {
        Some(ApiContent::Text(text)) => {
            assert_eq!(text, "");
        }
        _ => panic!("Expected Text content"),
    }
    assert!(messages[0].reasoning_content.as_ref().unwrap().is_empty());
}

// ########################################
// INTEGRATION TESTS
// ########################################

// Helper to create a mock stream from vec of chunks
fn mock_stream(
    chunks: Vec<GetChatCompletionChunk>,
) -> impl Stream<Item = Result<GetChatCompletionChunk, Status>> {
    stream::iter(chunks.into_iter().map(Ok))
}

fn make_simple_chunk(
    index: i32,
    reasoning: Option<&str>,
    content: Option<&str>,
) -> GetChatCompletionChunk {
    GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: Some(Delta {
                reasoning_content: reasoning.unwrap_or("").to_string(),
                content: content.unwrap_or("").to_string(),
                role: 0,
                tool_calls: vec![],
                encrypted_content: String::new(),
                citations: vec![],
            }),
            logprobs: None,
            finish_reason: FinishReason::ReasonInvalid as i32,
            index,
        }],
        created: None,
        model: "model".to_string(),
        system_fingerprint: String::new(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }
}

fn make_finish_chunk(index: i32) -> GetChatCompletionChunk {
    GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: None,
            logprobs: None,
            finish_reason: FinishReason::ReasonStop as i32,
            index,
        }],
        created: None,
        model: "model".to_string(),
        system_fingerprint: String::new(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }
}

// Test case 1: Basic stream with reasoning and content tokens
#[tokio::test]
async fn test_process_basic_stream() {
    let chunks = vec![
        make_simple_chunk(0, Some("Reasoning token 1"), Some("")),
        make_simple_chunk(0, Some(""), Some("Content token 1")),
        make_finish_chunk(0),
    ];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();

    let reasoning_tokens: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ReasoningToken(_, t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(reasoning_tokens, ["Reasoning token 1"]);

    let content_tokens: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ContentToken(_, t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(content_tokens, ["Content token 1"]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReasoningComplete(_)))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ContentComplete(_)))
    );
}

// Test case 2: Multi-output with finish reasons
#[tokio::test]
async fn test_process_multi_output() {
    let chunks = vec![
        // Chunk 1: Output 0 starts reasoning, Output 1 starts content
        GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![
                CompletionOutputChunk {
                    delta: Some(Delta {
                        reasoning_content: "Reason 1".to_string(),
                        content: "".to_string(),
                        role: 0,
                        tool_calls: vec![],
                        encrypted_content: "".to_string(),
                        citations: vec![],
                    }),
                    logprobs: None,
                    finish_reason: FinishReason::ReasonInvalid as i32,
                    index: 0,
                },
                CompletionOutputChunk {
                    delta: Some(Delta {
                        reasoning_content: "".to_string(),
                        content: "Content 1".to_string(),
                        role: 0,
                        tool_calls: vec![],
                        encrypted_content: "".to_string(),
                        citations: vec![],
                    }),
                    logprobs: None,
                    finish_reason: FinishReason::ReasonInvalid as i32,
                    index: 1,
                },
            ],
            created: None,
            model: "test_model".to_string(),
            system_fingerprint: "fp".to_string(),
            usage: None,
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        },
        // Chunk 2: Finishes
        GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![
                CompletionOutputChunk {
                    delta: None,
                    logprobs: None,
                    finish_reason: FinishReason::ReasonStop as i32,
                    index: 0,
                },
                CompletionOutputChunk {
                    delta: None,
                    logprobs: None,
                    finish_reason: FinishReason::ReasonStop as i32,
                    index: 1,
                },
            ],
            created: None,
            model: "test_model".to_string(),
            system_fingerprint: "fp".to_string(),
            usage: Some(SamplingUsage {
                completion_tokens: 1,
                reasoning_tokens: 1,
                prompt_tokens: 0,
                total_tokens: 2,
                prompt_text_tokens: 0,
                cached_prompt_text_tokens: 0,
                prompt_image_tokens: 0,
                num_sources_used: 0,
                server_side_tools_used: vec![],
                cost_in_usd_ticks: None,
            }),
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        },
    ];

    let (collected, events) = run_process(mock_stream(chunks)).await.unwrap();
    assert_eq!(collected.len(), 2);

    let reasoning_indices: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ReasoningComplete(ctx) => Some(ctx.output_index),
            _ => None,
        })
        .collect();
    assert_eq!(reasoning_indices.len(), 1);
    assert!(reasoning_indices.contains(&0));

    let content_indices: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ContentComplete(ctx) => Some(ctx.output_index),
            _ => None,
        })
        .collect();
    assert_eq!(content_indices.len(), 1);
    assert!(content_indices.contains(&1));
}

// Test case 3: Empty stream
#[tokio::test]
async fn test_process_empty_stream() {
    let (chunks, events) = run_process(mock_stream(vec![])).await.unwrap();
    assert!(chunks.is_empty());
    assert!(matches!(events.as_slice(), [Event::Complete]));
}

// Test case 4: Stream with tool calls
#[tokio::test]
async fn test_process_with_tool_calls() {
    let chunks = vec![GetChatCompletionChunk {
        id: "test_id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: Some(Delta {
                reasoning_content: "".to_string(),
                content: "".to_string(),
                role: 0,
                tool_calls: vec![
                    ToolCall {
                        id: "tool1".to_string(),
                        r#type: ToolCallType::ClientSideTool as i32,
                        status: 0,
                        error_message: None,
                        index: None,
                        tool: Some(tool_call::Tool::Function(FunctionCall {
                            name: "client_tool".to_string(),
                            arguments: "{}".to_string(),
                        })),
                    },
                    ToolCall {
                        id: "tool2".to_string(),
                        r#type: ToolCallType::WebSearchTool as i32,
                        status: 0,
                        error_message: None,
                        index: None,
                        tool: Some(tool_call::Tool::Function(FunctionCall {
                            name: "web_search".to_string(),
                            arguments: "{}".to_string(),
                        })),
                    },
                ],
                encrypted_content: "".to_string(),
                citations: vec![],
            }),
            logprobs: None,
            finish_reason: FinishReason::ReasonToolCalls as i32,
            index: 0,
        }],
        created: None,
        model: "test_model".to_string(),
        system_fingerprint: "fp".to_string(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }];

    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    let client_calls: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ClientToolCalls(_, c) => Some(c),
            _ => None,
        })
        .flatten()
        .collect();
    let server_calls: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ServerToolCalls(_, c) => Some(c),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(client_calls.len(), 1);
    assert_eq!(client_calls[0].id, "tool1");
    assert_eq!(server_calls.len(), 1);
    assert_eq!(server_calls[0].id, "tool2");
}

// Test case 5: Stream error handling
#[tokio::test]
async fn test_process_error_in_stream() {
    let error_stream = stream::iter(vec![
        Ok(GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![],
            created: None,
            model: "test_model".to_string(),
            system_fingerprint: "fp".to_string(),
            usage: None,
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        }),
        Err(Status::internal("Simulated error")),
    ]);
    let (tx, rx) = mpsc::unbounded();
    let result = process(error_stream, tx).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Simulated error");
    let events = collect_events(rx).await;
    assert!(matches!(events.last(), Some(Event::Error(_))));
}

// Test case 6: Error as first stream item
#[tokio::test]
async fn test_process_error_as_first_item() {
    let error_first = stream::iter(vec![Err(Status::cancelled("Cancelled"))]);
    let (tx, rx) = mpsc::unbounded();
    let result = process(error_first, tx).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Cancelled");
    let events = collect_events(rx).await;
    let Event::Error(status) = &events[0] else {
        panic!("expected Event::Error");
    };
    assert_eq!(status.message(), "Cancelled");
}

// Test case 7: Chunk events for every chunk
#[tokio::test]
async fn test_process_emits_chunk_per_grpc_chunk() {
    let chunks = vec![
        make_simple_chunk(0, Some("r1"), Some("c1")),
        make_simple_chunk(0, Some(""), Some("c2")),
        make_finish_chunk(0),
    ];
    let (collected, events) = run_process(mock_stream(chunks)).await.unwrap();
    assert_eq!(collected.len(), 3);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Chunk(_)))
            .count(),
        3
    );
}

// Test case 8: reasoning/content start once per output
#[tokio::test]
async fn test_process_reasoning_start_and_content_start_once() {
    let chunks = vec![
        make_simple_chunk(0, Some("reason1"), Some("")),
        make_simple_chunk(0, Some("reason2"), Some("")),
        make_simple_chunk(0, Some(""), Some("content1")),
        make_simple_chunk(0, Some(""), Some("content2")),
        make_finish_chunk(0),
    ];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::ReasoningStart(_)))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::ContentStart(_)))
            .count(),
        1
    );
}

// Test case 9: usage event from last chunk
#[tokio::test]
async fn test_process_emits_usage() {
    let usage = SamplingUsage {
        completion_tokens: 10,
        reasoning_tokens: 5,
        prompt_tokens: 1,
        total_tokens: 16,
        prompt_text_tokens: 0,
        cached_prompt_text_tokens: 0,
        prompt_image_tokens: 0,
        num_sources_used: 0,
        server_side_tools_used: vec![],
        cost_in_usd_ticks: None,
    };
    let mut last = make_finish_chunk(0);
    last.usage = Some(usage.clone());
    let chunks = vec![make_simple_chunk(0, Some("r"), Some("c")), last];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    let usage_ev = events.iter().find_map(|e| match e {
        Event::Usage(u) => u.as_ref(),
        _ => None,
    });
    assert_eq!(usage_ev.unwrap().completion_tokens, 10);
}

// Test case 10: citations event
#[tokio::test]
async fn test_process_emits_citations() {
    let mut last = make_finish_chunk(0);
    last.citations = vec!["https://a.com".to_string(), "https://b.com".to_string()];
    let chunks = vec![make_simple_chunk(0, Some("r"), Some("c")), last];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    let urls = events.iter().find_map(|e| match e {
        Event::Citations(c) => Some(c.as_slice()),
        _ => None,
    });
    assert_eq!(
        urls,
        Some(&["https://a.com".to_string(), "https://b.com".to_string()][..])
    );
}

// Test case 10b: complete event
#[tokio::test]
async fn test_process_emits_complete() {
    let chunks = vec![
        make_simple_chunk(0, Some("r"), Some("c")),
        make_finish_chunk(0),
    ];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::Complete)));
}

// Test case 10c: complete on empty stream
#[tokio::test]
async fn test_process_emits_complete_empty_stream() {
    let (_chunks, events) = run_process(mock_stream(vec![])).await.unwrap();
    assert!(matches!(events.as_slice(), [Event::Complete]));
}

// Test case 11: inline citations event
#[tokio::test]
async fn test_process_emits_inline_citations() {
    let mut chunk = make_simple_chunk(0, Some(""), Some("text"));
    if let Some(ref mut d) = chunk.outputs[0].delta {
        d.citations = vec![InlineCitation {
            id: "1".to_string(),
            start_index: 0,
            end_index: 4,
            citation: None,
            ..Default::default()
        }];
    }
    let chunks = vec![chunk, make_finish_chunk(0)];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    let ids: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::InlineCitations(_, c) => {
                Some(c.iter().map(|x| x.id.as_str()).collect::<Vec<_>>())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(ids, ["1"]);
}

// Test case 12: Chunk with empty outputs still collected
#[tokio::test]
async fn test_process_chunk_with_empty_outputs_collected() {
    let chunks = vec![GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![],
        created: None,
        model: "m".to_string(),
        system_fingerprint: "".to_string(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }];
    let (result, _events) = run_process(mock_stream(chunks.clone())).await.unwrap();
    assert_eq!(result.len(), 1);
    assert!(result[0].outputs.is_empty());
}

// Test case 13: reasoning_complete once
#[tokio::test]
async fn test_process_reasoning_complete_once() {
    let chunks = vec![
        make_simple_chunk(0, Some("r1"), Some("")),
        make_simple_chunk(0, Some("r2"), Some("")),
        make_simple_chunk(0, Some("r3"), Some("")),
        make_finish_chunk(0),
    ];
    let (_chunks, events) = run_process(mock_stream(chunks)).await.unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::ReasoningComplete(_)))
            .count(),
        1
    );
}

// ########################################
// process() event emission tests
// ########################################

#[tokio::test]
async fn test_process_emits_events_basic_stream() {
    let chunks = vec![
        make_simple_chunk(0, Some("r1"), Some("")),
        make_simple_chunk(0, Some(""), Some("c1")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks.clone()), tx).await.unwrap();
    let events = collect_events(rcv).await;

    let chunk_events = events
        .iter()
        .filter(|e| matches!(e, Event::Chunk(_)))
        .count();
    assert_eq!(chunk_events, 3, "expected 3 Chunk events");

    let reasoning_tokens: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::ReasoningToken(_, t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(reasoning_tokens, ["r1"]);

    let content_tokens: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::ContentToken(_, t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(content_tokens, ["c1"]);

    assert!(events.iter().any(|e| matches!(e, Event::ReasoningStart(_))));
    assert!(events.iter().any(|e| matches!(e, Event::ContentStart(_))));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReasoningComplete(_)))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ContentComplete(_)))
    );
}

#[tokio::test]
async fn test_process_emits_empty_stream() {
    let chunks: Vec<GetChatCompletionChunk> = vec![];
    let (tx, rcv) = mpsc::unbounded();
    let result = process(mock_stream(chunks), tx).await.unwrap();
    assert!(result.is_empty());
    let events = collect_events(rcv).await;
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::Complete));
}

#[tokio::test]
async fn test_process_emits_chunk_per_chunk() {
    let chunks = vec![
        make_simple_chunk(0, Some("r"), Some("c")),
        make_simple_chunk(0, Some(""), Some("")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let chunk_count = events
        .iter()
        .filter(|e| matches!(e, Event::Chunk(_)))
        .count();
    assert_eq!(chunk_count, 3);
}

#[tokio::test]
async fn test_process_emits_usage_event() {
    let usage = SamplingUsage {
        completion_tokens: 10,
        reasoning_tokens: 5,
        prompt_tokens: 1,
        total_tokens: 16,
        prompt_text_tokens: 0,
        cached_prompt_text_tokens: 0,
        prompt_image_tokens: 0,
        num_sources_used: 0,
        server_side_tools_used: vec![],
        cost_in_usd_ticks: None,
    };
    let mut last = make_finish_chunk(0);
    last.usage = Some(usage);
    let chunks = vec![make_simple_chunk(0, Some("r"), Some("c")), last];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let usage_ev = events.iter().find_map(|e| match e {
        Event::Usage(u) => u.as_ref(),
        _ => None,
    });
    assert!(usage_ev.is_some());
    assert_eq!(usage_ev.unwrap().completion_tokens, 10);
    assert_eq!(usage_ev.unwrap().reasoning_tokens, 5);
}

#[tokio::test]
async fn test_process_emits_citations_event() {
    let mut last = make_finish_chunk(0);
    last.citations = vec!["https://a.com".to_string(), "https://b.com".to_string()];
    let chunks = vec![make_simple_chunk(0, Some("r"), Some("c")), last];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let citations_ev = events.iter().find_map(|e| match e {
        Event::Citations(c) => Some(c),
        _ => None,
    });
    let expected = vec!["https://a.com".to_string(), "https://b.com".to_string()];
    assert_eq!(
        citations_ev.map(|v| v.as_slice()),
        Some(expected.as_slice())
    );
}

#[tokio::test]
async fn test_process_emits_inline_citations_event() {
    let mut chunk = make_simple_chunk(0, Some(""), Some("text"));
    if let Some(ref mut d) = chunk.outputs[0].delta {
        d.citations = vec![InlineCitation {
            id: "cite-1".to_string(),
            start_index: 0,
            end_index: 4,
            citation: None,
            ..Default::default()
        }];
    }
    let chunks = vec![chunk, make_finish_chunk(0)];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let inline_ev = events.iter().find_map(|e| match e {
        Event::InlineCitations(_, c) => Some(c),
        _ => None,
    });
    assert!(inline_ev.is_some());
    assert_eq!(inline_ev.unwrap().len(), 1);
    assert_eq!(inline_ev.unwrap()[0].id, "cite-1");
}

#[tokio::test]
async fn test_process_emits_client_and_server_tool_calls() {
    let chunks = vec![GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: Some(Delta {
                reasoning_content: "".to_string(),
                content: "".to_string(),
                role: 0,
                tool_calls: vec![
                    ToolCall {
                        id: "client-1".to_string(),
                        r#type: ToolCallType::ClientSideTool as i32,
                        status: 0,
                        error_message: None,
                        index: None,
                        tool: Some(tool_call::Tool::Function(FunctionCall {
                            name: "client_tool".to_string(),
                            arguments: "{}".to_string(),
                        })),
                    },
                    ToolCall {
                        id: "server-1".to_string(),
                        r#type: ToolCallType::WebSearchTool as i32,
                        status: 0,
                        error_message: None,
                        index: None,
                        tool: Some(tool_call::Tool::Function(FunctionCall {
                            name: "web_search".to_string(),
                            arguments: "{}".to_string(),
                        })),
                    },
                ],
                encrypted_content: "".to_string(),
                citations: vec![],
            }),
            logprobs: None,
            finish_reason: FinishReason::ReasonToolCalls as i32,
            index: 0,
        }],
        created: None,
        model: "m".to_string(),
        system_fingerprint: String::new(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let client_ev = events.iter().find_map(|e| match e {
        Event::ClientToolCalls(_, c) => Some(c),
        _ => None,
    });
    let server_ev = events.iter().find_map(|e| match e {
        Event::ServerToolCalls(_, c) => Some(c),
        _ => None,
    });
    assert!(client_ev.is_some());
    assert_eq!(client_ev.unwrap().len(), 1);
    assert_eq!(client_ev.unwrap()[0].id, "client-1");
    assert!(server_ev.is_some());
    assert_eq!(server_ev.unwrap().len(), 1);
    assert_eq!(server_ev.unwrap()[0].id, "server-1");
}

#[tokio::test]
async fn test_process_emits_reasoning_start_once_per_output() {
    let chunks = vec![
        make_simple_chunk(0, Some("reason1"), Some("")),
        make_simple_chunk(0, Some("reason2"), Some("")),
        make_simple_chunk(0, Some(""), Some("content1")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let reasoning_starts = events
        .iter()
        .filter(|e| matches!(e, Event::ReasoningStart(_)))
        .count();
    assert_eq!(reasoning_starts, 1);
}

#[tokio::test]
async fn test_process_emits_content_start_once_per_output() {
    let chunks = vec![
        make_simple_chunk(0, Some("r"), Some("")),
        make_simple_chunk(0, Some(""), Some("c1")),
        make_simple_chunk(0, Some(""), Some("c2")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let content_starts = events
        .iter()
        .filter(|e| matches!(e, Event::ContentStart(_)))
        .count();
    assert_eq!(content_starts, 1);
}

#[tokio::test]
async fn test_process_emits_reasoning_complete_once() {
    let chunks = vec![
        make_simple_chunk(0, Some("r1"), Some("")),
        make_simple_chunk(0, Some("r2"), Some("")),
        make_simple_chunk(0, Some("r3"), Some("")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let reasoning_complete = events
        .iter()
        .filter(|e| matches!(e, Event::ReasoningComplete(_)))
        .count();
    assert_eq!(reasoning_complete, 1);
}

#[tokio::test]
async fn test_process_emits_content_complete_once() {
    let chunks = vec![
        make_simple_chunk(0, Some("r"), Some("c1")),
        make_simple_chunk(0, Some(""), Some("c2")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let content_complete = events
        .iter()
        .filter(|e| matches!(e, Event::ContentComplete(_)))
        .count();
    assert_eq!(content_complete, 1);
}

#[tokio::test]
async fn test_process_emits_multi_output_output_context() {
    let chunks = vec![
        GetChatCompletionChunk {
            id: "id".to_string(),
            outputs: vec![
                CompletionOutputChunk {
                    delta: Some(Delta {
                        reasoning_content: "r0".to_string(),
                        content: "".to_string(),
                        role: 0,
                        tool_calls: vec![],
                        encrypted_content: String::new(),
                        citations: vec![],
                    }),
                    logprobs: None,
                    finish_reason: FinishReason::ReasonInvalid as i32,
                    index: 0,
                },
                CompletionOutputChunk {
                    delta: Some(Delta {
                        reasoning_content: "".to_string(),
                        content: "c1".to_string(),
                        role: 0,
                        tool_calls: vec![],
                        encrypted_content: String::new(),
                        citations: vec![],
                    }),
                    logprobs: None,
                    finish_reason: FinishReason::ReasonInvalid as i32,
                    index: 1,
                },
            ],
            created: None,
            model: "m".to_string(),
            system_fingerprint: String::new(),
            usage: None,
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        },
        GetChatCompletionChunk {
            id: "id".to_string(),
            outputs: vec![
                CompletionOutputChunk {
                    delta: None,
                    logprobs: None,
                    finish_reason: FinishReason::ReasonStop as i32,
                    index: 0,
                },
                CompletionOutputChunk {
                    delta: None,
                    logprobs: None,
                    finish_reason: FinishReason::ReasonStop as i32,
                    index: 1,
                },
            ],
            created: None,
            model: "m".to_string(),
            system_fingerprint: String::new(),
            usage: None,
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        },
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let reasoning_ctx: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ReasoningToken(ctx, _) => Some(ctx.output_index),
            _ => None,
        })
        .collect();
    let content_ctx: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::ContentToken(ctx, _) => Some(ctx.output_index),
            _ => None,
        })
        .collect();
    assert_eq!(reasoning_ctx, [0]);
    assert_eq!(content_ctx, [1]);
}

#[tokio::test]
async fn test_process_emits_chunk_empty_outputs() {
    let chunks = vec![GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![],
        created: None,
        model: "m".to_string(),
        system_fingerprint: String::new(),
        usage: None,
        citations: vec![],
        debug_output: None,
        output_files: vec![],
        service_tier: 0,
    }];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks.clone()), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let chunk_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Chunk(c) => Some(c.outputs.len()),
            _ => None,
        })
        .collect();
    assert_eq!(chunk_events, [0]);
    assert!(!events.iter().any(|e| matches!(e, Event::ReasoningStart(_))));
    assert!(!events.iter().any(|e| matches!(e, Event::ContentStart(_))));
}

#[tokio::test]
async fn test_process_emits_stream_error_returns_err() {
    let error_stream = stream::iter(vec![
        Ok(GetChatCompletionChunk {
            id: "id".to_string(),
            outputs: vec![],
            created: None,
            model: "m".to_string(),
            system_fingerprint: String::new(),
            usage: None,
            citations: vec![],
            debug_output: None,
            output_files: vec![],
            service_tier: 0,
        }),
        Err(Status::internal("simulated error")),
    ]);
    let (tx, rcv) = mpsc::unbounded();
    let result = process(error_stream, tx).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "simulated error");
    let events = collect_events(rcv).await;
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], Event::Chunk(_)));
    assert!(matches!(events[1], Event::Error(_)));
    let Event::Error(status) = &events[1] else {
        panic!("expected Event::Error");
    };
    assert_eq!(status.message(), "simulated error");
    assert!(!events.iter().any(|e| matches!(e, Event::Complete)));
}

#[tokio::test]
async fn test_process_emits_event_order_chunk_before_phase_events() {
    let chunks = vec![
        make_simple_chunk(0, Some("r"), Some("")),
        make_finish_chunk(0),
    ];
    let (tx, rcv) = mpsc::unbounded();
    let _ = process(mock_stream(chunks), tx).await.unwrap();
    let events = collect_events(rcv).await;
    let first_chunk_pos = events.iter().position(|e| matches!(e, Event::Chunk(_)));
    let reasoning_start_pos = events
        .iter()
        .position(|e| matches!(e, Event::ReasoningStart(_)));
    assert!(first_chunk_pos.is_some());
    assert!(reasoning_start_pos.is_some());
    assert!(first_chunk_pos.unwrap() < reasoning_start_pos.unwrap());
}
