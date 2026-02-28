// tests/integration.rs

use std::sync::{Arc, Mutex};
use futures::stream::{self, Stream};
use tonic::{Status, Streaming};
use xai_sdk::chat::stream::{process, Consumer, OutputContext, PhaseStatus, BoxFuture};
use xai_sdk::xai_api::{GetChatCompletionChunk, CompletionOutputChunk, Delta, ToolCall, ToolCallType, InlineCitation, SamplingUsage, FinishReason};

// Helper to create a mock stream from vec of chunks
fn mock_stream(chunks: Vec<GetChatCompletionChunk>) -> impl Stream<Item = Result<GetChatCompletionChunk, Status>> {
    stream::iter(chunks.into_iter().map(Ok))
}

// Test case 1: Basic single-output stream with reasoning and content
#[tokio::test]
async fn test_process_basic_stream() {
    let chunks = vec![
        GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![CompletionOutputChunk {
                delta: Some(Delta {
                    reasoning_content: "Reasoning token 1".to_string(),
                    content: "".to_string(),
                    role: 0,
                    tool_calls: vec![],
                    encrypted_content: "".to_string(),
                    citations: vec![],
                }),
                logprobs: None,
                finish_reason: FinishReason::ReasonInvalid as i32,
                index: 0,
            }],
            created: None,
            model: "test_model".to_string(),
            system_fingerprint: "fp".to_string(),
            usage: None,
            citations: vec![],
            debug_output: None,
        },
        GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![CompletionOutputChunk {
                delta: Some(Delta {
                    reasoning_content: "".to_string(),
                    content: "Content token 1".to_string(),
                    role: 0,
                    tool_calls: vec![],
                    encrypted_content: "".to_string(),
                    citations: vec![],
                }),
                logprobs: None,
                finish_reason: FinishReason::ReasonInvalid as i32,
                index: 0,
            }],
            created: None,
            model: "test_model".to_string(),
            system_fingerprint: "fp".to_string(),
            usage: None,
            citations: vec![],
            debug_output: None,
        },
        // Final chunk with usage and finish
        GetChatCompletionChunk {
            id: "test_id".to_string(),
            outputs: vec![CompletionOutputChunk {
                delta: None,
                logprobs: None,
                finish_reason: FinishReason::ReasonStop as i32,
                index: 0,
            }],
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
            }),
            citations: vec![],
            debug_output: None,
        },
    ];

    let stream = mock_stream(chunks.clone());

    // Setup consumer with callbacks to collect data
    let collected_reasoning_tokens = Arc::new(Mutex::new(Vec::new()));
    let collected_content_tokens = Arc::new(Mutex::new(Vec::new()));
    let reasoning_complete_fired = Arc::new(Mutex::new(false));
    let content_complete_fired = Arc::new(Mutex::new(false));

    let collected_reasoning_tokens_clone = collected_reasoning_tokens.clone();
    let collected_content_tokens_clone = collected_content_tokens.clone();
    let reasoning_complete_fired_clone = reasoning_complete_fired.clone();
    let content_complete_fired_clone = content_complete_fired.clone();

    let mut consumer = Consumer::new();
    consumer.on_reasoning_token = Some(Box::new(move |_: &OutputContext, token: &str| {
        let collected = collected_reasoning_tokens_clone.clone();
        Box::pin(async move {
            if !token.is_empty() {
                collected.lock().unwrap().push(token.to_string());
            }
        })
    }));
    consumer.on_content_token = Some(Box::new(move |_: &OutputContext, token: &str| {
        let collected = collected_content_tokens_clone.clone();
        Box::pin(async move {
            if !token.is_empty() {
                collected.lock().unwrap().push(token.to_string());
            }
        })
    }));
    consumer.on_reasoning_complete = Some(Box::new(move |_: &OutputContext| {
        let fired = reasoning_complete_fired_clone.clone();
        Box::pin(async move {
            *fired.lock().unwrap() = true;
        })
    }));
    consumer.on_content_complete = Some(Box::new(move |_: &OutputContext| {
        let fired = content_complete_fired_clone.clone();
        Box::pin(async move {
            *fired.lock().unwrap() = true;
        })
    }));

    let result = process(mock_stream(chunks.clone()), consumer).await.unwrap();

    // Assertions
    assert_eq!(result.len(), chunks.len());  // All chunks collected

    let reasoning_tokens = collected_reasoning_tokens.lock().unwrap();
    assert_eq!(reasoning_tokens.len(), 1);
    assert_eq!(reasoning_tokens[0], "Reasoning token 1");

    let content_tokens = collected_content_tokens.lock().unwrap();
    assert_eq!(content_tokens.len(), 1);
    assert_eq!(content_tokens[0], "Content token 1");

    assert!(*reasoning_complete_fired.lock().unwrap());
    assert!(*content_complete_fired.lock().unwrap());
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
            }),
            citations: vec![],
            debug_output: None,
        },
    ];

    let stream = mock_stream(chunks.clone());

    // Consumer to track phase completions
    let reasoning_completes = Arc::new(Mutex::new(Vec::new()));
    let content_completes = Arc::new(Mutex::new(Vec::new()));

    let reasoning_completes_clone = reasoning_completes.clone();
    let content_completes_clone = content_completes.clone();

    let mut consumer = Consumer::new();
    consumer.on_reasoning_complete = Some(Box::new(move |ctx: &OutputContext| {
        let c = reasoning_completes_clone.clone();
        Box::pin(async move { c.lock().unwrap().push(ctx.output_index); })
    }));
    consumer.on_content_complete = Some(Box::new(move |ctx: &OutputContext| {
        let c = content_completes_clone.clone();
        Box::pin(async move { c.lock().unwrap().push(ctx.output_index); })
    }));

    let result = process(stream, consumer).await.unwrap();

    // Assertions
    assert_eq!(result.len(), chunks.len());

    let reasoning_fired = reasoning_completes.lock().unwrap();
    assert_eq!(reasoning_fired.len(), 1);  // Only output 0 had reasoning
    assert!(reasoning_fired.contains(&0));

    let content_fired = content_completes.lock().unwrap();
    assert_eq!(content_fired.len(), 2);  // Both outputs had content (output 1 started content, output 0 implicitly complete)
    assert!(content_fired.contains(&0));
    assert!(content_fired.contains(&1));
}

// Test case 3: Empty stream
#[tokio::test]
async fn test_process_empty_stream() {
    let stream = mock_stream(vec![]);
    let consumer = Consumer::new();
    let result = process(stream, consumer).await.unwrap();
    assert!(result.is_empty());  // No chunks collected
}

// Test case 4: Stream with tool calls
#[tokio::test]
async fn test_process_with_tool_calls() {
    let chunks = vec![
        GetChatCompletionChunk {
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
                            status: 0,  // InProgress
                            error_message: None,
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
        },
    ];

    let stream = mock_stream(chunks.clone());

    let client_tools_called = Arc::new(Mutex::new(Vec::new()));
    let server_tools_called = Arc::new(Mutex::new(Vec::new()));

    let client_tools_clone = client_tools_called.clone();
    let server_tools_clone = server_tools_called.clone();

    let mut consumer = Consumer::new();
    consumer.on_client_tool_calls = Some(Box::new(move |_: &OutputContext, calls: &[ToolCall]| {
        let c = client_tools_clone.clone();
        Box::pin(async move { c.lock().unwrap().extend(calls.to_vec()); })
    }));
    consumer.on_server_tool_calls = Some(Box::new(move |_: &OutputContext, calls: &[ToolCall]| {
        let c = server_tools_clone.clone();
        Box::pin(async move { c.lock().unwrap().extend(calls.to_vec()); })
    }));

    let result = process(stream, consumer).await.unwrap();

    assert_eq!(result.len(), 1);

    let client_calls = client_tools_called.lock().unwrap();
    assert_eq!(client_calls.len(), 1);
    assert_eq!(client_calls[0].id, "tool1");

    let server_calls = server_tools_called.lock().unwrap();
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
        }),
        Err(Status::internal("Simulated error")),
    ]);

    let consumer = Consumer::new();
    let result = process(error_stream, consumer).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message(), "Simulated error");
}
