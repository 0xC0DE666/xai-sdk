use anyhow::{Context, Result};
use std::env;
use tonic::metadata::MetadataValue;
use tonic::{Request, Streaming};
use xai_sdk::{
    Content, GetChatCompletionChunk, GetCompletionsRequest, Message, MessageRole, chat, content,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create the client
    let mut client = chat::client::new(&api_key).await?;

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
        n: Some(1),
        ..Default::default()
    });

    // Add authentication header
    let token =
        MetadataValue::try_from(format!("Bearer {}", api_key)).context("Invalid API key format")?;
    request.metadata_mut().insert("authorization", token);

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: Quote Tyler Durden.");
    println!("🤖 Model: grok-3-latest");
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
