use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use xai_sdk::api::{
    Content, Function, GetChatCompletionChunk, GetCompletionsRequest, InlineCitation, Message,
    MessageRole, Tool, ToolCall, ToolCallStatus, ToolCallType, XSearch, content,
};
use xai_sdk::chat;
use xai_sdk::chat::stream::{Consumer, OutputContext};
use xai_sdk::{Request, Streaming};

const WRITE_FILE: &str = "write_file";

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated chat client
    let mut client = chat::client::new(&api_key).await?;

    let prompt = "What are the last two tweets from @elonmusk and @tsoding? Write their tweets to 'musk.txt' and 'tsoding.txt'";
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

    // Create write_file tool
    let write_file_tool = write_file_tool();

    let request = Request::new(GetCompletionsRequest {
        model: model.to_string(),
        messages,
        tools: vec![xsearch_tool, write_file_tool],
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
            let consumer = Consumer::new()
                .on_reasoning_token(|ctx: &OutputContext, token: &str| {
                    print!("{token}");
                    io::stdout().flush().unwrap();
                    async {}
                })
                .on_reasoning_complete(move |ctx: &OutputContext| {
                    println!("on_reasoning_complete -------------------------------------------------\n");
                    async {}
                })
                // on_content_token: Print content in real-time
                .on_content_token(move |_ctx: &OutputContext, token: &str| {
                    print!("{token}");
                    io::stdout().flush().unwrap();
                    async {}
                })
                // on_content_complete: New line after content
                .on_content_complete(move |ctx: &OutputContext| {
                    println!(
                        "on_content_complete -------------------------------------------------\n"
                    );
                    async {}
                })
                // on_inline_citations: Show citations inline
                .on_inline_citations(move |_ctx: &OutputContext, citations: &[InlineCitation]| {
                    let citations = citations.to_vec();
                    async move {
                        if !citations.is_empty() {
                            println!("\n📚 Found {} inline citation(s):", citations.len());
                            for citation in &citations {
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
                    }
                })
                .on_client_tool_calls(move |ctx: &OutputContext, tool_calls: &[ToolCall]| {
                    let output_index = ctx.output_index;
                    let tool_calls = tool_calls.to_vec();
                    // AI TODO: call print_tool_calls for client side tools
                    // check for a write file tool call as per definition, parse the args and execute
                    // the tool call.
                    async move {
                        println!("on_client_tool_calls -------------------------------------------------\n");
                    }
                })
                // on_server_tool_calls: Show server tool call details in real-time
                .on_server_tool_calls(move |ctx: &OutputContext, tool_calls: &[ToolCall]| {
                    let output_index = ctx.output_index;
                    let tool_calls = tool_calls.to_vec();
                    // AI TODO: extract this display logic to function print_tool_calls(calls: &[ToolCall])
                    // then call print_tool_calls, move all the code into this context (above async block)
                    async move {
                        dbg!(output_index);
                        println!("\n🔧 Tool Call(s) Detected:");
                        for tool_call in &tool_calls {
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
                        println!("on_server_tool_calls -------------------------------------------------\n");
                    }
                })
                // on_usage: Show final statistics
                .on_usage(move |usage: &xai_sdk::api::SamplingUsage| {
                    let prompt_tokens = usage.prompt_tokens;
                    let completion_tokens = usage.completion_tokens;
                    let reasoning_tokens = usage.reasoning_tokens;
                    let total_tokens = usage.total_tokens;
                    async move {
                        println!("\n📊 Token Usage:");
                        println!("  Prompt: {} tokens", prompt_tokens);
                        println!("  Completion: {} tokens", completion_tokens);
                        println!("  Reasoning: {} tokens", reasoning_tokens);
                        println!("  Total: {} tokens", total_tokens);
                        println!(
                            "on_usage -------------------------------------------------\n"
                        );
                    }
                })
                // on_citations: Show final citations
                .on_citations(move |citations: &[String]| {
                    let citations = citations.to_vec();
                    async move {
                        if !citations.is_empty() {
                            println!("\n🔗 Source Citations:");
                            for (i, citation) in citations.iter().enumerate() {
                                println!("  {}. {}", i + 1, citation);
                            }
                        }
                        println!(
                            "on_citations -------------------------------------------------\n"
                        );
                    }
                });

            // Process the stream
            match chat::stream::process(stream, consumer).await {
                Ok(chunks) => {
                    println!("\n✅ Stream completed ({} chunks processed)", chunks.len());

                    // Write all chunks to chunks.txt in dbg format
                    let chunks_debug = format!("{:#?}", chunks);
                    write_file(PathBuf::from("debug/chunks.txt"), chunks_debug)
                        .context("Failed to write chunks to chunks.txt")?;
                    println!("📝 All chunks written to chunks.txt");

                    // let res = chat::stream::assemble(chunks);
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

/// Writes content to a file at the specified path.
// AI TODO: refactor this to use tokio fs
fn write_file(path: PathBuf, content: String) -> Result<()> {
    std::fs::write(&path, content).context(format!("Failed to write file: {:?}", path))?;
    Ok(())
}

/// Creates a tool definition for the write_file function.
fn write_file_tool() -> Tool {
    let def = Function {
        name: WRITE_FILE.into(),
        description: "Write a file to disk.".into(),
        parameters: json!({
            "type": "object",
            "properties": json!({
                "name": json!({
                    "description": "The name of the file.",
                    "type": "string",
                }),
                "content": json!({
                    "description": "The content to be written.",
                    "type": "string",
                })
            }),
            "required": json!(["name", "content"]),
        })
        .to_string(),
        strict: true,
    };

    Tool {
        tool: Some(xai_sdk::api::tool::Tool::Function(def)),
    }
}
