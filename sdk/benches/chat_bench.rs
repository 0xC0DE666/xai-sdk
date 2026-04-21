//! Benchmarks for `chat::stream`: `assemble`, `process` (minimal and with callbacks),
//! and `Consumer::with_sink` (event-based path).

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use futures::StreamExt;
use futures::channel::mpsc;
use futures::stream::{self, Stream};
use std::sync::Arc;
use tokio::runtime::Runtime;
use xai_sdk::Status;
use xai_sdk::api::{
    CompletionOutputChunk, Delta, FinishReason, GetChatCompletionChunk, SamplingUsage,
};
use xai_sdk::chat::stream::{Consumer, Event, assemble, process};

fn make_content_chunk(index: i32, token: &str) -> GetChatCompletionChunk {
    GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: Some(Delta {
                reasoning_content: String::new(),
                content: token.to_string(),
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
        usage: Some(SamplingUsage {
            completion_tokens: 1,
            reasoning_tokens: 0,
            prompt_tokens: 0,
            total_tokens: 1,
            prompt_text_tokens: 0,
            cached_prompt_text_tokens: 0,
            prompt_image_tokens: 0,
            num_sources_used: 0,
            server_side_tools_used: vec![],
        }),
        citations: vec![],
        debug_output: None,
    }
}

/// Builds `n` content chunks plus one finish chunk (single output, index 0).
fn build_chunks(n: usize) -> Vec<GetChatCompletionChunk> {
    let mut chunks = Vec::with_capacity(n + 1);
    for _ in 0..n {
        chunks.push(make_content_chunk(0, "x"));
    }
    chunks.push(make_finish_chunk(0));
    chunks
}

fn make_reasoning_chunk(index: i32, token: &str) -> GetChatCompletionChunk {
    GetChatCompletionChunk {
        id: "id".to_string(),
        outputs: vec![CompletionOutputChunk {
            delta: Some(Delta {
                reasoning_content: token.to_string(),
                content: String::new(),
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
    }
}

/// Builds a stream with both reasoning and content tokens: one reasoning chunk, `n` content chunks, then finish.
/// Exercises ReasoningStart, ReasoningToken, ContentStart, ContentToken, ReasoningComplete, ContentComplete.
fn build_chunks_reasoning_then_content(n: usize) -> Vec<GetChatCompletionChunk> {
    let mut chunks = Vec::with_capacity(n + 2);
    chunks.push(make_reasoning_chunk(0, "think"));
    for _ in 0..n {
        chunks.push(make_content_chunk(0, "x"));
    }
    chunks.push(make_finish_chunk(0));
    chunks
}

fn mock_stream(
    chunks: Vec<GetChatCompletionChunk>,
) -> impl Stream<Item = Result<GetChatCompletionChunk, Status>> + Send + Unpin + 'static {
    stream::iter(chunks.into_iter().map(Ok))
}

fn bench_assemble(c: &mut Criterion) {
    let mut group = c.benchmark_group("assemble");
    for size in [10, 100, 1_000, 10_000] {
        let chunks = build_chunks(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("assemble_{}_chunks", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let cloned = chunks.clone();
                    black_box(assemble(cloned))
                });
            },
        );
    }
    group.finish();
}

fn bench_process(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("process");
    for size in [10, 100, 1_000, 10_000] {
        let chunks = build_chunks(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("process_{}_chunks_minimal_consumer", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let stream = mock_stream(chunks.clone());
                    let consumer = Consumer::new();
                    rt.block_on(async { black_box(process(stream, consumer).await) })
                });
            },
        );
    }
    group.finish();
}

fn bench_process_with_simple_callbacks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("process");
    for size in [10, 100, 1_000] {
        let chunks = build_chunks(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("process_{}_chunks_with_token_count_callbacks", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let stream = mock_stream(chunks.clone());
                    let token_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                    let token_count_clone = token_count.clone();
                    let mut consumer = Consumer::new();
                    consumer.on_content_token = Some(Box::new(move |_ctx, token: &str| {
                        let c = token_count_clone.clone();
                        let t = token.to_string();
                        Box::pin(async move {
                            if !t.is_empty() {
                                c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        })
                    }));
                    let result = rt.block_on(async { process(stream, consumer).await });
                    let _ = result.expect("process should succeed in bench");
                    black_box(token_count.load(std::sync::atomic::Ordering::Relaxed));
                });
            },
        );
    }
    group.finish();
}

/// Process stream with `Consumer::with_sink(tx)` and drain all events from the receiver.
/// Measures the event-based path (sink send + drain).
fn bench_process_with_sink(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("process_with_sink");
    for size in [10, 100, 1_000, 10_000] {
        let chunks = build_chunks(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("with_sink_{}_chunks", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let (tx, mut rcv) = mpsc::unbounded();
                    let consumer = Consumer::with_sink(tx);
                    let stream = mock_stream(chunks.clone());
                    let result = rt.block_on(async {
                        let r = process(stream, consumer).await;
                        let events: Vec<Event> = rcv.by_ref().collect().await;
                        (r, events)
                    });
                    let (r, events) = result;
                    let _ = r.expect("process should succeed in bench");
                    black_box(events.len());
                });
            },
        );
    }
    group.finish();
}

/// Process with with_sink on a stream that has both reasoning and content (more event types).
fn bench_process_with_sink_reasoning_and_content(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("process_with_sink");
    for size in [10, 100, 1_000] {
        let chunks = build_chunks_reasoning_then_content(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("with_sink_{}_chunks_reasoning_and_content", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let (tx, mut rcv) = mpsc::unbounded();
                    let consumer = Consumer::with_sink(tx);
                    let stream = mock_stream(chunks.clone());
                    let result = rt.block_on(async {
                        let r = process(stream, consumer).await;
                        let events: Vec<Event> = rcv.by_ref().collect().await;
                        (r, events)
                    });
                    let (r, events) = result;
                    let _ = r.expect("process should succeed in bench");
                    black_box(events.len());
                });
            },
        );
    }
    group.finish();
}

/// Process with with_sink and count only ContentToken events (simulates a handler that
/// only cares about content tokens).
fn bench_process_with_sink_count_content_tokens(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("process_with_sink");
    for size in [10, 100, 1_000] {
        let chunks = build_chunks(size);
        let count = chunks.len();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            format!("with_sink_{}_chunks_count_content_tokens", count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let (tx, mut rcv) = mpsc::unbounded();
                    let consumer = Consumer::with_sink(tx);
                    let stream = mock_stream(chunks.clone());
                    let (result, content_count) = rt.block_on(async {
                        let r = process(stream, consumer).await;
                        let mut n = 0usize;
                        while let Some(ev) = rcv.next().await {
                            if let Event::ContentToken(_, _) = ev {
                                n += 1;
                            }
                        }
                        (r, n)
                    });
                    let _ = result.expect("process should succeed in bench");
                    black_box(content_count);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_assemble,
    bench_process,
    bench_process_with_simple_callbacks,
    bench_process_with_sink,
    bench_process_with_sink_reasoning_and_content,
    bench_process_with_sink_count_content_tokens
);
criterion_main!(benches);
