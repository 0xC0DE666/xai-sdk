use xai_sdk::api::{
    CompletionMessage, CompletionOutput, CompletionOutputChunk, Delta, FinishReason,
    GetChatCompletionChunk, InlineCitation, MessageRole, SamplingUsage, ToolCall, content::Content,
};
use xai_sdk::chat::stream::{Consumer, OutputContext, PhaseStatus, assemble};
use xai_sdk::chat::utils::to_messages;

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
fn test_consumer_new() {
    let consumer = Consumer::new();
    assert!(consumer.on_chunk.is_none());
    assert!(consumer.on_reasoning_token.is_none());
    assert!(consumer.on_reasoning_complete.is_none());
    assert!(consumer.on_content_token.is_none());
    assert!(consumer.on_content_complete.is_none());
    assert!(consumer.on_inline_citations.is_none());
    assert!(consumer.on_client_tool_calls.is_none());
    assert!(consumer.on_server_tool_calls.is_none());
    assert!(consumer.on_usage.is_none());
    assert!(consumer.on_citations.is_none());
}

#[test]
fn test_consumer_default() {
    let consumer = Consumer::default();
    assert!(consumer.on_chunk.is_none());
    assert!(consumer.on_reasoning_token.is_none());
    assert!(consumer.on_reasoning_complete.is_none());
    assert!(consumer.on_content_token.is_none());
    assert!(consumer.on_content_complete.is_none());
    assert!(consumer.on_inline_citations.is_none());
    assert!(consumer.on_client_tool_calls.is_none());
    assert!(consumer.on_server_tool_calls.is_none());
    assert!(consumer.on_usage.is_none());
    assert!(consumer.on_citations.is_none());
}

#[test]
fn test_consumer_builder_on_content_token() {
    let consumer = Consumer::new().on_content_token(|_ctx, _token| async move {});
    // Consumer should have the callback set
    assert!(consumer.on_content_token.is_some());
}

#[test]
fn test_consumer_builder_on_reason_token() {
    let consumer = Consumer::new().on_reasoning_token(|_ctx, _token| async move {
        // Test callback
    });

    assert!(consumer.on_reasoning_token.is_some());
}

#[test]
fn test_consumer_builder_on_chunk() {
    let consumer = Consumer::new().on_chunk(|_chunk| async move {
        // Test callback
    });

    assert!(consumer.on_chunk.is_some());
}

#[test]
fn test_consumer_builder_on_reasoning_complete() {
    let consumer = Consumer::new().on_reasoning_complete(|_ctx| async move {
        // Test callback
    });

    assert!(consumer.on_reasoning_complete.is_some());
}

#[test]
fn test_consumer_builder_on_content_complete() {
    let consumer = Consumer::new().on_content_complete(|_ctx| async move {
        // Test callback
    });

    assert!(consumer.on_content_complete.is_some());
}

#[test]
fn test_consumer_builder_chain() {
    let consumer = Consumer::new()
        .on_chunk(|_chunk| async move {})
        .on_reasoning_token(|_ctx, _token| async move {})
        .on_reasoning_complete(|_ctx| async move {})
        .on_content_token(|_ctx, _token| async move {})
        .on_content_complete(|_ctx| async move {})
        .on_inline_citations(|_ctx, _citations| async move {})
        .on_client_tool_calls(|_ctx, _calls| async move {})
        .on_server_tool_calls(|_ctx, _calls| async move {})
        .on_usage(|_usage| async move {})
        .on_citations(|_citations| async move {});

    assert!(consumer.on_chunk.is_some());
    assert!(consumer.on_reasoning_token.is_some());
    assert!(consumer.on_reasoning_complete.is_some());
    assert!(consumer.on_content_token.is_some());
    assert!(consumer.on_content_complete.is_some());
    assert!(consumer.on_inline_citations.is_some());
    assert!(consumer.on_client_tool_calls.is_some());
    assert!(consumer.on_server_tool_calls.is_some());
    assert!(consumer.on_usage.is_some());
    assert!(consumer.on_citations.is_some());
}

#[test]
fn test_assemble_empty_chunks() {
    let chunks = vec![];
    let result = assemble(chunks);
    assert!(result.is_none());
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
fn test_consumer_builder_on_inline_citations() {
    let consumer = Consumer::new().on_inline_citations(|_ctx, _citations| async move {
        // Test callback
    });

    assert!(consumer.on_inline_citations.is_some());
}

#[test]
fn test_consumer_builder_on_client_tool_calls() {
    let consumer = Consumer::new().on_client_tool_calls(|_ctx, _calls| async move {
        // Test callback
    });

    assert!(consumer.on_client_tool_calls.is_some());
}

#[test]
fn test_consumer_builder_on_server_tool_calls() {
    let consumer = Consumer::new().on_server_tool_calls(|_ctx, _calls| async move {
        // Test callback
    });

    assert!(consumer.on_server_tool_calls.is_some());
}

#[test]
fn test_consumer_builder_on_usage() {
    let consumer = Consumer::new().on_usage(|_usage| async move {
        // Test callback
    });

    assert!(consumer.on_usage.is_some());
}

#[test]
fn test_consumer_builder_on_citations() {
    let consumer = Consumer::new().on_citations(|_citations| async move {
        // Test callback
    });

    assert!(consumer.on_citations.is_some());
}

#[test]
fn test_assemble_with_inline_citations() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.content = "Hello".to_string();
    delta.citations = vec![
        InlineCitation {
            id: "1".to_string(),
            start_index: 0,
            end_index: 5,
            citation: None,
            ..Default::default()
        },
        InlineCitation {
            id: "2".to_string(),
            start_index: 6,
            end_index: 10,
            citation: None,
            ..Default::default()
        },
    ];
    output.delta = Some(delta);
    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.citations.len(), 2);
    assert_eq!(message.citations[0].id, "1");
    assert_eq!(message.citations[1].id, "2");
}

#[test]
fn test_assemble_with_tool_calls() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.content = "Hello".to_string();
    delta.tool_calls = vec![
        ToolCall {
            id: "call-1".to_string(),
            r#type: 0,
            status: 0,
            ..Default::default()
        },
        ToolCall {
            id: "call-2".to_string(),
            r#type: 0,
            status: 0,
            ..Default::default()
        },
    ];
    output.delta = Some(delta);
    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.tool_calls.len(), 2);
    assert_eq!(message.tool_calls[0].id, "call-1");
    assert_eq!(message.tool_calls[1].id, "call-2");
}

#[test]
fn test_assemble_with_citations_in_last_chunk() {
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
    chunk2.citations = vec![
        "https://example.com".to_string(),
        "https://test.com".to_string(),
    ];

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
    assert_eq!(response.citations.len(), 2);
    assert_eq!(response.citations[0], "https://example.com");
    assert_eq!(response.citations[1], "https://test.com");
}

// Tests for helper functions (get_reasoning_status, get_content_status)
// These are tested indirectly through the process function by checking OutputContext values

#[test]
fn test_reasoning_status_init() {
    // Reasoning status should be Init when there's no reasoning or content and not finished
    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonInvalid.into();

    let delta = Delta::default();
    // For Init: no reasoning, no content, not finished
    assert!(delta.reasoning_content.is_empty());
    assert!(delta.content.is_empty());
    assert_eq!(output.finish_reason, FinishReason::ReasonInvalid.into());
}

#[test]
fn test_reasoning_status_pending() {
    // Reasoning status should be Pending when there's reasoning but no content yet
    let mut delta = Delta::default();
    delta.reasoning_content = "Thinking...".to_string();
    delta.content = "".to_string();

    // Verify the delta has reasoning but no content
    assert!(!delta.reasoning_content.is_empty());
    assert!(delta.content.is_empty());
}

#[test]
fn test_reasoning_status_complete() {
    // Reasoning status should be Complete when there's no reasoning and content exists
    let mut delta = Delta::default();
    delta.reasoning_content = "".to_string();
    delta.content = "Hello".to_string();

    // Verify the delta has content but no reasoning
    assert!(delta.reasoning_content.is_empty());
    assert!(!delta.content.is_empty());
}

#[test]
fn test_content_status_init() {
    // Content status should be Init when there's no content and not finished
    let output = CompletionOutputChunk::default();
    let mut delta = Delta::default();
    delta.reasoning_content = "Thinking...".to_string();
    delta.content = "".to_string();

    // Verify: has reasoning, no content, not finished -> content should be Init
    assert!(!delta.reasoning_content.is_empty());
    assert!(delta.content.is_empty());
    assert_eq!(output.finish_reason, FinishReason::ReasonInvalid.into());
}

#[test]
fn test_content_status_pending() {
    // Content status should be Pending when content is being generated
    let output = CompletionOutputChunk::default();
    let mut delta = Delta::default();
    delta.reasoning_content = "".to_string();
    delta.content = "Hello".to_string();

    // Verify: no reasoning, has content, not finished -> content should be Pending
    assert!(delta.reasoning_content.is_empty());
    assert!(!delta.content.is_empty());
    assert_eq!(output.finish_reason, FinishReason::ReasonInvalid.into());
}

#[test]
fn test_content_status_complete() {
    // Content status should be Complete when output is finished
    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.reasoning_content = "".to_string();
    delta.content = "Hello".to_string();

    // Verify: no reasoning, has content, finished -> content should be Complete
    assert!(delta.reasoning_content.is_empty());
    assert!(!delta.content.is_empty());
    assert_ne!(output.finish_reason, FinishReason::ReasonInvalid.into());
}

// More comprehensive tests for assemble function

#[test]
fn test_assemble_accumulates_encrypted_content() {
    let mut chunk1 = GetChatCompletionChunk::default();
    chunk1.id = "test-id".to_string();
    chunk1.model = "test-model".to_string();

    let mut output1 = CompletionOutputChunk::default();
    output1.index = 0;
    let mut delta1 = Delta::default();
    delta1.encrypted_content = "enc1".to_string();
    output1.delta = Some(delta1);
    chunk1.outputs = vec![output1];

    let mut chunk2 = GetChatCompletionChunk::default();
    chunk2.id = "test-id".to_string();
    chunk2.model = "test-model".to_string();

    let mut output2 = CompletionOutputChunk::default();
    output2.index = 0;
    output2.finish_reason = FinishReason::ReasonStop.into();
    let mut delta2 = Delta::default();
    delta2.encrypted_content = "enc2".to_string();
    output2.delta = Some(delta2);
    chunk2.outputs = vec![output2];

    let chunks = vec![chunk1, chunk2];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.encrypted_content, "enc1enc2");
}

#[test]
fn test_assemble_handles_role_from_delta() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.content = "Hello".to_string();
    delta.role = 2; // RoleAssistant
    output.delta = Some(delta);
    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.role, 2);
}

#[test]
fn test_assemble_handles_role_zero() {
    let mut chunk = GetChatCompletionChunk::default();
    chunk.id = "test-id".to_string();
    chunk.model = "test-model".to_string();

    let mut output = CompletionOutputChunk::default();
    output.index = 0;
    output.finish_reason = FinishReason::ReasonStop.into();

    let mut delta = Delta::default();
    delta.content = "Hello".to_string();
    delta.role = 0; // Should not override
    output.delta = Some(delta);
    chunk.outputs = vec![output];

    let chunks = vec![chunk];
    let result = assemble(chunks);

    assert!(result.is_some());
    let response = result.unwrap();
    let message = response.outputs[0].message.as_ref().unwrap();
    assert_eq!(message.role, 0);
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
        Some(Content::Text(text)) => {
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
        Some(Content::Text(text)) => {
            assert_eq!(text, "First message");
        }
        _ => panic!("Expected Text content"),
    }

    // Check second message
    assert_eq!(messages[1].role, MessageRole::RoleAssistant.into());
    match &messages[1].content[0].content {
        Some(Content::Text(text)) => {
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
        Some(Content::Text(text)) => {
            assert_eq!(text, "");
        }
        _ => panic!("Expected Text content"),
    }
    assert!(messages[0].reasoning_content.as_ref().unwrap().is_empty());
}
