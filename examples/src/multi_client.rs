use anyhow::{Context, Result};
use xai_sdk::Request;
use xai_sdk::xai_api::{
    Content, GetCompletionsRequest, GetModelRequest, Message, MessageRole, SampleTextRequest,
    content,
};
use xai_sdk::{chat, common, models, sample};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key =
        std::env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    println!("🚀 xAI SDK Modular Client Example");
    println!("=================================\n");

    // Create shared channel
    let channel = common::channel::new().await?;

    // Create authenticated clients for different services
    let mut models_client = models::client::with_channel(channel.clone(), &api_key);
    let mut sample_client = sample::client::with_channel(channel.clone(), &api_key);
    let mut chat_client = chat::client::with_channel(channel.clone(), &api_key);

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
        name: "grok-code-fast".to_string(),
    });
    let model_response = models_client.get_language_model(model_request).await?;
    println!("Language model: {:#?}\n", model_response.into_inner());

    // Generate text using sample service
    println!("✍️  Generating text...");
    let sample_request = Request::new(SampleTextRequest {
        prompt: vec!["Write a haiku about Rust programming".to_string()],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.8),
        ..Default::default()
    });
    let sample_response = sample_client.sample_text(sample_request).await?;
    println!("{}", sample_response.into_inner().choices[0].text);

    // Chat completion
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
        model: "grok-3-latest".to_string(),
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
