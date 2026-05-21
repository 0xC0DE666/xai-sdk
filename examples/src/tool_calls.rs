use anyhow::{Context, Result};
use futures::StreamExt;
use futures::channel::mpsc;
use serde_json::json;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use xai_sdk::api::{
    Content, Function, GetChatCompletionChunk, GetCompletionsRequest, Message,
    MessageRole, Tool, ToolCall, ToolCallStatus, ToolCallType, XSearch, content,
};
use xai_sdk::chat;
use xai_sdk::chat::stream::{Event, process};
use xai_sdk::{Request, Streaming};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    let mut client = chat::client::new(&api_key).await?;

    let prompt = "What where the last two tweets from @elonmusk and @tsoding? Write their tweets with sources to 'musk.txt' and 'tsoding.txt'.";
    let model = "grok-4-latest";

    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text(prompt.into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];

    let xsearch = XSearch::default();
    let xsearch_tool = Tool {
        tool: Some(xai_sdk::api::tool::Tool::XSearch(xsearch)),
    };

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

    match client.get_completion_chunk(request).await {
        Ok(response) => {
            let stream: Streaming<GetChatCompletionChunk> = response.into_inner();
            let (tx, mut rx) = mpsc::unbounded();

            match process(stream, tx).await {
                Ok(chunks) => {
                    while let Some(ev) = rx.next().await {
                        handle_event(ev).await?;
                    }
                    println!("\n✅ Done ({} chunks)", chunks.len());
                    let chunks_debug = format!("{:#?}", chunks);
                    write_file(PathBuf::from("debug/chunks.txt"), chunks_debug)
                        .await
                        .context("Failed to write debug/chunks.txt")?;
                    println!("📝 Chunks saved to debug/chunks.txt");
                }
                Err(e) => {
                    while let Some(ev) = rx.next().await {
                        if let Event::Error(status) = ev {
                            eprintln!("\n❌ Stream error: {}", status);
                        }
                    }
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

async fn handle_event(ev: Event) -> Result<()> {
    match ev {
        Event::ReasoningToken(_, token) => {
            print!("{token}");
            io::stdout().flush().unwrap();
        }
        Event::ReasoningComplete(_) => println!(),
        Event::ContentToken(_, token) => {
            print!("{token}");
            io::stdout().flush().unwrap();
        }
        Event::ContentComplete(_) => println!(),
        Event::InlineCitations(_, citations) => {
            if !citations.is_empty() {
                println!("\n📚 {} inline citation(s)", citations.len());
                for c in &citations {
                    if let Some(ref data) = c.citation {
                        println!("  • [{}] {:?}", c.id, data);
                    } else {
                        println!("  • [{}] (no data)", c.id);
                    }
                }
            }
        }
        Event::ClientToolCalls(_, tool_calls) => {
            print_tool_calls(&tool_calls);
            for (path, content) in client_write_paths(&tool_calls) {
                if let Err(e) = write_file(path.clone(), content).await {
                    eprintln!("Failed to write {:?}: {}", path, e);
                }
            }
        }
        Event::ServerToolCalls(_, tool_calls) => print_tool_calls(&tool_calls),
        Event::Usage(usage) => {
            if let Some(usage) = usage {
                println!(
                    "\n📊 Tokens: prompt={} completion={} reasoning={} total={}",
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.reasoning_tokens,
                    usage.total_tokens
                );
            }
        }
        Event::Citations(citations) => {
            if !citations.is_empty() {
                println!("\n🔗 Sources:");
                for (i, c) in citations.iter().enumerate() {
                    println!("  {}. {}", i + 1, c);
                }
            }
        }
        Event::Error(status) => eprintln!("\n❌ Stream error: {}", status),
        Event::Complete => {}
        _ => {}
    }
    Ok(())
}

fn client_write_paths(tool_calls: &[ToolCall]) -> Vec<(PathBuf, String)> {
    tool_calls
        .iter()
        .filter_map(|tc| {
            let Some(xai_sdk::api::tool_call::Tool::Function(f)) = &tc.tool else {
                return None;
            };
            if f.name != WRITE_FILE {
                return None;
            }
            let args: serde_json::Value = serde_json::from_str(&f.arguments).ok()?;
            let name = args.get("name")?.as_str()?.to_string();
            let content = args.get("content")?.as_str()?.to_string();
            Some((PathBuf::from(name), content))
        })
        .collect()
}

fn print_tool_calls(tool_calls: &[ToolCall]) {
    if tool_calls.is_empty() {
        return;
    }
    println!("\n🔧 Tool call(s):");
    for tool_call in tool_calls {
        let tool_type = match ToolCallType::try_from(tool_call.r#type) {
            Ok(ToolCallType::XSearchTool) => "XSearch (Twitter/X)",
            Ok(ToolCallType::WebSearchTool) => "WebSearch",
            Ok(ToolCallType::CodeExecutionTool) => "CodeExecution",
            Ok(ToolCallType::CollectionsSearchTool) => "CollectionsSearch",
            Ok(ToolCallType::McpTool) => "MCP",
            Ok(ToolCallType::AttachmentSearchTool) => "AttachmentSearch",
            Ok(ToolCallType::ClientSideTool) => "Client-side function",
            _ => "Unknown",
        };
        let status = match ToolCallStatus::try_from(tool_call.status) {
            Ok(ToolCallStatus::InProgress) => "⏳ In progress",
            Ok(ToolCallStatus::Completed) => "✅ Completed",
            Ok(ToolCallStatus::Incomplete) => "⚠️ Incomplete",
            Ok(ToolCallStatus::Failed) => "❌ Failed",
            _ => "❓ Unknown",
        };
        println!("  ┌─ {} (id: {}, {})", tool_type, tool_call.id, status);
        if let Some(xai_sdk::api::tool_call::Tool::Function(f)) = &tool_call.tool {
            println!("  │  Function: {}", f.name);
            if !f.arguments.is_empty() {
                let args = if f.arguments.len() > 80 {
                    format!("{}...", &f.arguments[..80])
                } else {
                    f.arguments.clone()
                };
                println!("  │  Arguments: {}", args);
            }
        }
        println!("  └─");
    }
}

const WRITE_FILE: &str = "write_file";

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

async fn write_file(path: PathBuf, content: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context(format!("Failed to create parent dir for {:?}", path))?;
    }
    tokio::fs::write(&path, content)
        .await
        .context(format!("Failed to write file: {:?}", path))?;
    Ok(())
}
