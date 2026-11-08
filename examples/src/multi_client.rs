use anyhow::{Context, Result};
use xai_sdk::api::{
    Content, GetCompletionsRequest, GetModelRequest, Message, MessageRole, content,
};
use xai_sdk::export::tonic::Request;
use xai_sdk::{chat, models, shared, XAI_API_KEY};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var(XAI_API_KEY)
        .with_context(|| format!("{XAI_API_KEY} environment variable must be set"))?;

    println!("🚀 xAI SDK Modular Client Example");
    println!("=================================\n");

    // Create shared channel
    let channel = shared::channel::new().await?;

    // Create authenticated clients for different services
    let mut models_client = models::client::with_channel(channel.clone(), &api_key);
    let mut chat_client = chat::client::with_channel(channel, &api_key);

    // List available models
    println!("📋 Listing available models...");
    let models_request = Request::new(());
    let models_response = models_client.list_language_models(models_request).await?;
    println!(
        "Available models: {:#?}\n",
        models_response.into_inner().models
    );

    // Get language model detail
    println!("📋 Language model details...");
    let model_request = Request::new(GetModelRequest {
        name: "grok-4.20".to_string(),
    });
    let model_response = models_client.get_language_model(model_request).await?;
    println!("Language model: {:#?}\n", model_response.into_inner());

    println!("💬 Chat completion...");
    let message = Message {
        role: MessageRole::RoleUser.into(),
        content: vec![Content {
            content: Some(content::Content::Text(
                "Explain Rust ownership in simple terms".to_string(),
            )),
        }],
        ..Default::default()
    };
    let chat_request = Request::new(GetCompletionsRequest {
        model: "grok-4.20".to_string(),
        messages: vec![message],
        ..Default::default()
    });
    let chat_response = chat_client.get_completion(chat_request).await?;
    println!(
        "{}",
        chat_response.into_inner().outputs[0]
            .message
            .as_ref()
            .unwrap()
            .content
    );

    println!("✅ All operations completed successfully!");

    Ok(())
}
