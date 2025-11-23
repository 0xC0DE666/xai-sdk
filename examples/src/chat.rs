use anyhow::{Context, Result};
use std::env;
use xai_sdk::chat;
use xai_sdk::xai_api::{
    Content, GetChatCompletionChunk, GetChatCompletionResponse, GetCompletionsRequest, Message,
    MessageRole, content,
};
use xai_sdk::{Request, Streaming};

const COMPLETE: &str = "--complete";
const STREAM: &str = "--stream";
const ASSEMBLE: &str = "--assemble";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect(); // Collects arguments into a vector

    if args.len() < 2 {
        help();
    }

    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    let arg = args[1].as_str();
    let _ = match arg {
        COMPLETE => complete(&api_key).await,
        STREAM => stream(&api_key).await,
        ASSEMBLE => assemble(&api_key).await,
        _ => {
            println!("[ERROR]: Unknown flag: {arg}");
            help();
            Ok(())
        }
    };

    Ok(())
}

fn help() {
    println!("Chat Examples");
    println!("  {}      - Run post complete example", COMPLETE);
    println!("  {}      - Run stream example", STREAM);
    println!("  {}      - Run assemble stream result example", ASSEMBLE);
}

async fn complete(api_key: &str) -> Result<()> {
    // Create authenticated chat client
    let mut client = chat::client::new(&api_key).await?;

    // Create the request
    let prompt = "Quote Hannibal Lectre.";
    let model = "grok-3";
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text(prompt.into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];
    let request = Request::new(GetCompletionsRequest {
        model: model.to_string(),
        messages,
        n: Some(1),
        ..Default::default()
    });

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: {prompt}");
    println!("🤖 Model: {model}");
    println!();

    // Make the API call - authentication is automatic!
    match client.get_completion(request).await {
        Ok(response) => {
            let sample_response: GetChatCompletionResponse = response.into_inner();

            println!("✅ Response received!");
            println!("🆔 Request ID: {}", sample_response.id);
            println!("🤖 Model used: {}", sample_response.model);
            println!("📊 Usage: {:?}", sample_response.usage);
            println!();

            // Display the generated text
            for (i, choice) in sample_response.choices.iter().enumerate() {
                println!("Choice {}:", i + 1);
                println!("{}", choice.message.clone().unwrap().content);
                println!("Finish reason: {:?}", choice.finish_reason);
            }
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn stream(api_key: &str) -> Result<()> {
    // Create the client
    let mut client = chat::client::new(&api_key).await?;

    // Create the request
    let prompt = "What is 35 + 34?";
    let model = "grok-3-mini";
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text(prompt.into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];
    let request = Request::new(GetCompletionsRequest {
        model: model.to_string(),
        messages,
        n: Some(1),
        ..Default::default()
    });

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: {prompt}");
    println!("🤖 Model: {model}");
    println!();

    // Make the API call
    match client.get_completion_chunk(request).await {
        Ok(response) => {
            let stream: Streaming<GetChatCompletionChunk> = response.into_inner();
            let consumer = chat::stream::Consumer::with_stdout();
            let _ = chat::stream::process(stream, consumer).await;
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn assemble(api_key: &str) -> Result<()> {
    // Create authenticated chat client
    let mut client = chat::client::new(&api_key).await?;

    // Create the request
    let prompt = "Quote a King 810 song.";
    let model = "grok-3-mini";
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text(prompt.into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];
    let request = Request::new(GetCompletionsRequest {
        model: model.to_string(),
        messages,
        n: Some(3),
        ..Default::default()
    });

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: {prompt}");
    println!("🤖 Model: {model}");
    println!();

    // Make the API call and collect all chunks - authentication is automatic!
    match client.get_completion_chunk(request).await {
        Ok(response) => {
            let stream: Streaming<GetChatCompletionChunk> = response.into_inner();

            // Process the stream and collect all chunks
            let consumer = chat::stream::Consumer::new();
            let chunks = chat::stream::process(stream, consumer).await?;

            println!("📦 Collected {} chunks from stream", chunks.len());

            // Assemble the chunks into a complete response
            if let Some(response) = chat::stream::assemble(chunks) {
                println!("\n🎯 Assembled Response:");
                println!("ID: {}", response.id);
                println!("Model: {}", response.model);
                println!("Created: {:?}", response.created);
                println!("System Fingerprint: {}", response.system_fingerprint);

                if let Some(usage) = &response.usage {
                    println!("Usage:");
                    println!("  - Completion tokens: {}", usage.completion_tokens);
                    println!("  - Reasoning tokens: {}", usage.reasoning_tokens);
                    println!("  - Prompt tokens: {}", usage.prompt_tokens);
                    println!("  - Total tokens: {}", usage.total_tokens);
                }

                if !response.citations.is_empty() {
                    println!("Citations: {:?}", response.citations);
                }

                println!("\n📝 Choices:");
                for (i, choice) in response.choices.iter().enumerate() {
                    println!(
                        "Choice {} (index: {}, finish_reason: {}):",
                        i, choice.index, choice.finish_reason
                    );

                    if let Some(message) = &choice.message {
                        println!("  Role: {}", message.role);
                        println!("  Content: {}", message.content);
                        if !message.reasoning_content.is_empty() {
                            println!("  Reasoning: {}", message.reasoning_content);
                        }
                        if !message.tool_calls.is_empty() {
                            println!("  Tool calls: {:?}", message.tool_calls);
                        }
                        println!();
                    }
                }
            } else {
                println!("❌ Failed to assemble response from chunks");
            }
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
