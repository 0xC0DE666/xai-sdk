use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use xai_sdk::api::{
    Content, GetChatCompletionChunk, GetCompletionsRequest, InlineCitation, Message, MessageRole,
    Tool, ToolCall, ToolCallStatus, ToolCallType, XSearch, content,
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

    let prompt =
        "What are the last two tweets from @elonmusk and @tsoding?";
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
        parallel_tool_calls: Some(true),
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

            // Create a locally scoped Consumer with real-time status updates
            let is_thinking = Arc::new(Mutex::new(false));
            let reasoning_started = Arc::new(Mutex::new(false));

            let consumer = Consumer::new()
                // on_reason_token: Show "Thinking..." indicator
                // .on_chunk(|chunk| {
                //     dbg!(chunk);
                // })
                .on_reason_token({
                    let is_thinking = is_thinking.clone();
                    let reasoning_started = reasoning_started.clone();
                    move |ctx: &OutputContext, _token: &str| {
                        if ctx.output_index == 0 {
                            let mut started = reasoning_started.lock().unwrap();
                            let mut thinking = is_thinking.lock().unwrap();
                            if !*started {
                                *started = true;
                                *thinking = true;
                                print!("\n💭 Thinking");
                                io::stdout().flush().unwrap();
                            } else if *thinking {
                                print!(".");
                                io::stdout().flush().unwrap();
                            }
                        }
                    }
                })
                // on_reasoning_complete: Clear thinking indicator
                .on_reasoning_complete({
                    let is_thinking = is_thinking.clone();
                    move |ctx: &OutputContext| {
                        let mut thinking = is_thinking.lock().unwrap();
                        if *thinking {
                            *thinking = false;
                            println!("\n");
                        }
                        dbg!(ctx);
                        println!("on_reasoning_complete -------------------------------------------------\n");
                    }
                })
                // on_content_token: Print content in real-time
                .on_content_token({
                    let is_thinking = is_thinking.clone();
                    move |ctx: &OutputContext, token: &str| {
                        // Clear thinking indicator if still showing
                        let mut thinking = is_thinking.lock().unwrap();
                        if *thinking {
                            *thinking = false;
                            println!("\n");
                        }
                        print!("{token}");
                        io::stdout().flush().unwrap();
                    }
                })
                // on_content_complete: New line after content
                .on_content_complete(move |ctx: &OutputContext| {
                    dbg!(ctx);
                    println!(
                        "on_content_complete -------------------------------------------------\n"
                    );
                })
                // on_inline_citations: Show citations inline
                .on_inline_citations(move |ctx: &OutputContext, citations: &[InlineCitation]| {
                    if !citations.is_empty() {
                        println!("\n📚 Found {} inline citation(s):", citations.len());
                        for citation in citations {
                            if let Some(ref citation_data) = citation.citation {
                                println!("  • [{}] {:?}", citation.id, citation_data);
                            } else {
                                println!("  • [{}] (no citation data)", citation.id);
                            }
                        }
                    }
                    println!(
                        "on_inline_citations -------------------------------------------------\n"
                    );
                })
                // on_tool_calls: Show tool call details in real-time
                .on_tool_calls(move |ctx: &OutputContext, tool_calls: &[ToolCall]| {
                    dbg!(ctx);
                    println!("\n🔧 Tool Call(s) Detected:");
                    for tool_call in tool_calls {
                        let tool_type = match ToolCallType::try_from(tool_call.r#type) {
                            Ok(ToolCallType::XSearchTool) => "XSearch (Twitter/X)",
                            Ok(ToolCallType::WebSearchTool) => "WebSearch",
                            Ok(ToolCallType::CodeExecutionTool) => "CodeExecution",
                            Ok(ToolCallType::CollectionsSearchTool) => "CollectionsSearch",
                            Ok(ToolCallType::McpTool) => "MCP",
                            Ok(ToolCallType::AttachmentSearchTool) => "AttachmentSearch",
                            Ok(ToolCallType::ClientSideTool) => "Client-Side Function",
                            _ => "Unknown",
                        };

                        let status = match ToolCallStatus::try_from(tool_call.status) {
                            Ok(ToolCallStatus::InProgress) => "⏳ In Progress",
                            Ok(ToolCallStatus::Completed) => "✅ Completed",
                            Ok(ToolCallStatus::Incomplete) => "⚠️  Incomplete",
                            Ok(ToolCallStatus::Failed) => "❌ Failed",
                            _ => "❓ Unknown",
                        };

                        println!("  ┌─ Tool: {}", tool_type);
                        println!("  │  ID: {}", tool_call.id);
                        println!("  │  Status: {}", status);

                        // Extract tool call details if it's a function call
                        if let Some(xai_sdk::api::tool_call::Tool::Function(function_call)) =
                            &tool_call.tool
                        {
                            println!("  │  Function: {}", function_call.name);
                            if !function_call.arguments.is_empty() {
                                // Truncate long arguments for display
                                let args_display = if function_call.arguments.len() > 100 {
                                    format!("{}...", &function_call.arguments[..100])
                                } else {
                                    function_call.arguments.clone()
                                };
                                println!("  │  Arguments: {}", args_display);
                            }
                        }
                        println!("  └─");
                    }
                    println!("on_tool_calls -------------------------------------------------\n");
                })
                // on_usage: Show final statistics
                .on_usage(move |usage: &xai_sdk::api::SamplingUsage| {
                    println!("\n📊 Token Usage:");
                    println!("  Prompt: {} tokens", usage.prompt_tokens);
                    println!("  Completion: {} tokens", usage.completion_tokens);
                    println!("  Reasoning: {} tokens", usage.reasoning_tokens);
                    println!("  Total: {} tokens", usage.total_tokens);
                    println!(
                        "on_usage -------------------------------------------------\n"
                    );
                })
                // on_citations: Show final citations
                .on_citations(move |citations: &[String]| {
                    if !citations.is_empty() {
                        println!("\n🔗 Source Citations:");
                        for (i, citation) in citations.iter().enumerate() {
                            println!("  {}. {}", i + 1, citation);
                        }
                    }
                    println!(
                        "on_citations -------------------------------------------------\n"
                    );
                });

            // Process the stream
            match chat::stream::process(stream, consumer).await {
                Ok(chunks) => {
                    println!("\n✅ Stream completed ({} chunks processed)", chunks.len());

                    // Write all chunks to chunks.txt in dbg format
                    let chunks_debug = format!("{:#?}", chunks);
                    fs::write("chunks.txt", chunks_debug)
                        .context("Failed to write chunks to chunks.txt")?;
                    println!("📝 All chunks written to chunks.txt");

                    let res = chat::stream::assemble(chunks);
                    dbg!(res);
                }
                Err(e) => {
                    eprintln!("\n❌ Error processing stream: {}", e);
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
