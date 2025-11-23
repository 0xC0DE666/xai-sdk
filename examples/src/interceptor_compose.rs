use anyhow::{Context, Result};
use std::env;
use xai_sdk::Request;
use xai_sdk::xai_api::{
    Content, GetChatCompletionResponse, GetCompletionsRequest, Message, MessageRole, content,
};
use xai_sdk::{chat, common};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Build interceptors: auth + some dummy metadata injectors
    let composed = common::interceptor::compose(vec![
        // Auth header (manually set to avoid type mismatch with helper)
        Box::new(move |mut r: Request<()>| {
            r.metadata_mut().insert(
                "authorization",
                format!("Bearer {}", api_key).parse().unwrap(),
            );
            Ok(r)
        }),
        // Add a trace id
        Box::new(|mut r: Request<()>| {
            r.metadata_mut()
                .insert("x-trace-id", "trace-abc123".parse().unwrap());
            Ok(r)
        }),
        // Add a tenant id
        Box::new(|mut r: Request<()>| {
            r.metadata_mut()
                .insert("x-tenant-id", "tenant-42".parse().unwrap());
            Ok(r)
        }),
        // Print the entire request (metadata and extensions)
        Box::new(|r: Request<()>| {
            println!(
                "\n===== Interceptor Debug - Outgoing Request =====\n{:#?}\n================================================\n",
                r
            );
            Ok(r)
        }),
    ]);

    // Create a chat client with the composed interceptor
    let mut client = chat::client::with_interceptor(composed).await?;

    // Prepare a minimal chat completion request
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text("Say hello briefly.".into()));
    let mut msg = Message::default();
    msg.role = MessageRole::RoleUser.into();
    msg.content = vec![cntnt];
    let messages = vec![msg];
    let request = Request::new(GetCompletionsRequest {
        model: "grok-3-latest".to_string(),
        messages,
        n: Some(1),
        ..Default::default()
    });

    // Call the API
    match client.get_completion(request).await {
        Ok(response) => {
            let resp: GetChatCompletionResponse = response.into_inner();
            println!(
                "✅ Response received!\nID: {} | Model: {}",
                resp.id, resp.model
            );
            if let Some(msg) = resp.choices.get(0).and_then(|c| c.message.as_ref()) {
                println!("🗨️  {}", msg.content);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
