use anyhow::{Context, Result};
use std::env;
use std::io::Write;
use std::sync::{Arc, Mutex};
use xai_sdk::api::{
    Content, GetChatCompletionChunk, GetCompletionsRequest, InlineCitation, Message, MessageRole,
    Tool, ToolCall, XSearch, content,
};
use xai_sdk::chat;
use xai_sdk::chat::stream::{Consumer, OutputContext};
use xai_sdk::{Request, Streaming};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated chat client
    let mut client = chat::client::new(&api_key).await?;

    // Create the request asking for Elon Musk's latest tweets
    let prompt =
        "What are Elon Musk's 5 latest tweets? Please provide the tweet text and timestamps.";
    let model = "grok-4-latest";

    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text(prompt.into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];

    // Create XSearch tool for searching X/Twitter
    let xsearch = XSearch::default();
    let xsearch_tool = Tool {
        tool: Some(xai_sdk::api::tool::Tool::XSearch(xsearch)),
    };

    let request = Request::new(GetCompletionsRequest {
        model: model.to_string(),
        messages,
        tools: vec![xsearch_tool],
        n: Some(1),
        ..Default::default()
    });

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: {prompt}");
    println!("🤖 Model: {model}");
    println!();

    // Make the streaming API call
    match client.get_completion_chunk(request).await {
        Ok(response) => {
            let stream: Streaming<GetChatCompletionChunk> = response.into_inner();

            // Create a Consumer with all callbacks implemented
            let consumer = create_full_consumer();

            // Process the stream
            match chat::stream::process(stream, consumer).await {
                Ok(chunks) => {
                    println!("\n✅ Stream processing completed!");
                    println!("📦 Total chunks received: {}", chunks.len());
                }
                Err(e) => {
                    eprintln!("❌ Error processing stream: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Creates a Consumer with all callbacks implemented to demonstrate
/// comprehensive stream processing capabilities.
fn create_full_consumer() -> Consumer<'static> {
    // Shared state for tracking stream progress
    let chunk_count = Arc::new(Mutex::new(0u32));
    let reasoning_tokens = Arc::new(Mutex::new(0u32));
    let content_tokens = Arc::new(Mutex::new(0u32));
    let reasoning_complete_count = Arc::new(Mutex::new(0u32));
    let content_complete_count = Arc::new(Mutex::new(0u32));
    let tool_calls_count = Arc::new(Mutex::new(0u32));
    let inline_citations_count = Arc::new(Mutex::new(0u32));

    Consumer::new()
        // on_chunk: Called once per complete chunk received
        .on_chunk({
            let chunk_count = chunk_count.clone();
            move |chunk: &GetChatCompletionChunk| {
                let mut count = chunk_count.lock().unwrap();
                *count += 1;
                if *count == 1 {
                    println!("📥 First chunk received (ID: {})", chunk.id);
                }
            }
        })
        // on_reason_token: Called for each reasoning token
        .on_reason_token({
            let reasoning_tokens = reasoning_tokens.clone();
            move |ctx: &OutputContext, _token: &str| {
                let mut count = reasoning_tokens.lock().unwrap();
                *count += 1;

                // Only print reasoning indicator for the first output to avoid clutter
                if ctx.output_index == 0 && *count <= 5 {
                    print!("💭");
                }
            }
        })
        // on_reasoning_complete: Called when reasoning phase completes
        .on_reasoning_complete({
            let reasoning_complete_count = reasoning_complete_count.clone();
            let reasoning_tokens = reasoning_tokens.clone();
            move |ctx: &OutputContext| {
                let mut count = reasoning_complete_count.lock().unwrap();
                *count += 1;
                let total_tokens = reasoning_tokens.lock().unwrap();
                println!(
                    "\n✅ Reasoning complete for output {} ({} reasoning tokens)",
                    ctx.output_index, *total_tokens
                );
            }
        })
        // on_content_token: Called for each content token
        .on_content_token({
            let content_tokens = content_tokens.clone();
            move |ctx: &OutputContext, token: &str| {
                let mut count = content_tokens.lock().unwrap();
                *count += 1;

                // Print content tokens for the first output
                if ctx.output_index == 0 {
                    print!("{token}");
                    std::io::stdout().flush().expect("Error flushing stdout");
                }
            }
        })
        // on_content_complete: Called when content phase completes
        .on_content_complete({
            let content_complete_count = content_complete_count.clone();
            let content_tokens = content_tokens.clone();
            move |ctx: &OutputContext| {
                let mut count = content_complete_count.lock().unwrap();
                *count += 1;
                let total_tokens = content_tokens.lock().unwrap();
                println!(
                    "\n✅ Content complete for output {} ({} content tokens)",
                    ctx.output_index, *total_tokens
                );
            }
        })
        // on_inline_citations: Called when inline citations are present
        .on_inline_citations({
            let inline_citations_count = inline_citations_count.clone();
            move |ctx: &OutputContext, citations: &[InlineCitation]| {
                let mut count = inline_citations_count.lock().unwrap();
                *count += 1;
                println!(
                    "\n📚 Inline citations found for output {} ({} citations)",
                    ctx.output_index,
                    citations.len()
                );
                for (i, citation) in citations.iter().enumerate() {
                    println!(
                        "  Citation {}: ID={}, range=[{}, {}]",
                        i + 1,
                        citation.id,
                        citation.start_index,
                        citation.end_index
                    );
                }
            }
        })
        // on_tool_calls: Called when tool calls are present
        .on_tool_calls({
            let tool_calls_count = tool_calls_count.clone();
            move |ctx: &OutputContext, tool_calls: &[ToolCall]| {
                let mut count = tool_calls_count.lock().unwrap();
                *count += 1;
                println!(
                    "\n🔧 Tool calls detected for output {} ({} tool calls)",
                    ctx.output_index,
                    tool_calls.len()
                );
                for (i, tool_call) in tool_calls.iter().enumerate() {
                    println!(
                        "  Tool call {}: ID={}, type={:?}, status={:?}",
                        i + 1,
                        tool_call.id,
                        tool_call.r#type,
                        tool_call.status
                    );
                }
            }
        })
        // on_usage: Called once on the last chunk with usage statistics
        .on_usage({
            let chunk_count = chunk_count.clone();
            let reasoning_complete_count = reasoning_complete_count.clone();
            let content_complete_count = content_complete_count.clone();
            let tool_calls_count = tool_calls_count.clone();
            let inline_citations_count = inline_citations_count.clone();
            move |usage: &xai_sdk::api::SamplingUsage| {
                println!("\n📊 Final Usage Statistics:");
                println!("  Prompt tokens: {}", usage.prompt_tokens);
                println!("  Completion tokens: {}", usage.completion_tokens);
                println!("  Reasoning tokens: {}", usage.reasoning_tokens);
                println!("  Total tokens: {}", usage.total_tokens);

                // Print summary of all callbacks
                let chunks = chunk_count.lock().unwrap();
                let reasoning_complete = reasoning_complete_count.lock().unwrap();
                let content_complete = content_complete_count.lock().unwrap();
                let tool_calls = tool_calls_count.lock().unwrap();
                let inline_citations = inline_citations_count.lock().unwrap();

                println!("\n📈 Callback Summary:");
                println!("  Chunks processed: {}", *chunks);
                println!("  Reasoning phases completed: {}", *reasoning_complete);
                println!("  Content phases completed: {}", *content_complete);
                println!("  Tool calls detected: {}", *tool_calls);
                println!("  Inline citations detected: {}", *inline_citations);
            }
        })
        // on_citations: Called once on the last chunk with citations
        .on_citations(move |citations: &[String]| {
            if !citations.is_empty() {
                println!("\n🔗 Citations from final chunk:");
                for (i, citation) in citations.iter().enumerate() {
                    println!("  {}: {}", i + 1, citation);
                }
            }
        })
}
