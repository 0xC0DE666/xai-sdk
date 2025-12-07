use xai_sdk::api::{
    CompletionOutputChunk, Delta, FinishReason, GetChatCompletionChunk, SamplingUsage,
};
use xai_sdk::chat::stream::{CompletionContext, Consumer, PhaseStatus, TokenContext, assemble};

#[test]
fn test_token_context_new() {
    let ctx = TokenContext::new(2, 1, PhaseStatus::Pending, PhaseStatus::Init);
    assert_eq!(ctx.total_choices, 2);
    assert_eq!(ctx.choice_index, 1);
    assert_eq!(ctx.reasoning_status, PhaseStatus::Pending);
    assert_eq!(ctx.content_status, PhaseStatus::Init);
}

#[test]
fn test_token_context_clone() {
    let ctx = TokenContext::new(1, 0, PhaseStatus::Complete, PhaseStatus::Complete);
    let cloned = ctx.clone();
    assert_eq!(cloned.total_choices, ctx.total_choices);
    assert_eq!(cloned.choice_index, ctx.choice_index);
    assert_eq!(cloned.reasoning_status, ctx.reasoning_status);
    assert_eq!(cloned.content_status, ctx.content_status);
}

#[test]
fn test_completion_context_new() {
    let ctx = CompletionContext::new(3, 2);
    assert_eq!(ctx.total_choices, 3);
    assert_eq!(ctx.choice_index, 2);
}

#[test]
fn test_completion_context_clone() {
    let ctx = CompletionContext::new(2, 1);
    let cloned = ctx.clone();
    assert_eq!(cloned.total_choices, ctx.total_choices);
    assert_eq!(cloned.choice_index, ctx.choice_index);
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
    assert!(consumer.on_content_token.is_none());
    assert!(consumer.on_content_complete.is_none());
    assert!(consumer.on_reason_token.is_none());
    assert!(consumer.on_reasoning_complete.is_none());
    assert!(consumer.on_chunk.is_none());
}

#[test]
fn test_consumer_default() {
    let consumer = Consumer::default();
    assert!(consumer.on_content_token.is_none());
    assert!(consumer.on_content_complete.is_none());
    assert!(consumer.on_reason_token.is_none());
    assert!(consumer.on_reasoning_complete.is_none());
    assert!(consumer.on_chunk.is_none());
}

#[test]
fn test_consumer_builder_on_content_token() {
    let mut content_tokens = Vec::new();
    let consumer = Consumer::new().on_content_token(move |_ctx, token| {
        content_tokens.push(token.to_string());
    });

    // Consumer should have the callback set
    assert!(consumer.on_content_token.is_some());
}

#[test]
fn test_consumer_builder_on_reason_token() {
    let consumer = Consumer::new().on_reason_token(|_ctx, _token| {
        // Test callback
    });

    assert!(consumer.on_reason_token.is_some());
}

#[test]
fn test_consumer_builder_on_chunk() {
    let consumer = Consumer::new().on_chunk(|_chunk| {
        // Test callback
    });

    assert!(consumer.on_chunk.is_some());
}

#[test]
fn test_consumer_builder_on_reasoning_complete() {
    let consumer = Consumer::new().on_reasoning_complete(|_ctx| {
        // Test callback
    });

    assert!(consumer.on_reasoning_complete.is_some());
}

#[test]
fn test_consumer_builder_on_content_complete() {
    let consumer = Consumer::new().on_content_complete(|_ctx| {
        // Test callback
    });

    assert!(consumer.on_content_complete.is_some());
}

#[test]
fn test_consumer_builder_chain() {
    let consumer = Consumer::new()
        .on_content_token(|_ctx, _token| {})
        .on_reason_token(|_ctx, _token| {})
        .on_chunk(|_chunk| {})
        .on_reasoning_complete(|_ctx| {})
        .on_content_complete(|_ctx| {});

    assert!(consumer.on_content_token.is_some());
    assert!(consumer.on_reason_token.is_some());
    assert!(consumer.on_chunk.is_some());
    assert!(consumer.on_reasoning_complete.is_some());
    assert!(consumer.on_content_complete.is_some());
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
