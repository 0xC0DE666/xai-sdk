use anyhow::{Context, Result};
use std::env;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Streaming};

use xai_sdk::chat;
use xai_sdk::chat_client::ChatClient;
use xai_sdk::{
    Content, GetChatCompletionChunk, GetCompletionsRequest, Message, MessageRole, content,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create the gRPC channel
    let channel = Channel::from_static("https://api.x.ai:443")
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await
        .context("Failed to connect to xAI API")?;

    // Create the client
    let mut client = ChatClient::new(channel);

    // Create the request
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text("Quote a King 810 song.".into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];
    let mut request = Request::new(GetCompletionsRequest {
        model: "grok-3-latest".to_string(),
        messages,
        n: Some(3),
        ..Default::default()
    });

    // Add authentication header
    let token =
        MetadataValue::try_from(format!("Bearer {}", api_key)).context("Invalid API key format")?;
    request.metadata_mut().insert("authorization", token);

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: Quote a King 810 song.");
    println!("🤖 Model: grok-3-latest");
    println!();

    // Make the API call and collect all chunks
    match client.get_completion_chunk(request).await {
        Ok(response) => {
            let stream: Streaming<GetChatCompletionChunk> = response.into_inner();

            // Process the stream and collect all chunks
            let consumer = chat::StreamConsumer::new();
            let chunks = chat::process_stream(stream, consumer).await?;

            println!("📦 Collected {} chunks from stream", chunks.len());

            // Assemble the chunks into a complete response
            if let Some(response) = chat::assemble_response(chunks) {
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
