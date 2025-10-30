use tonic::Request;
use xai_sdk::{
    Content, GetCompletionsRequest, Message, MessageRole, SampleTextRequest, chat, content, models,
    sample,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = std::env::var("XAI_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string());

    println!("🚀 xAI SDK Modular Client Example");
    println!("=================================\n");

    // Create authenticated clients for different services
    let mut models_client = models::client::new(&api_key).await?;
    let mut sample_client = sample::client::new(&api_key).await?;
    let mut chat_client = chat::client::new(&api_key).await?;

    // List available models
    println!("📋 Listing available models...");
    let models_request = Request::new(());
    let models_response = models_client.list_language_models(models_request).await?;
    println!(
        "Available models: {:?}\n",
        models_response.into_inner().models
    );

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
        chat_response.into_inner().choices[0]
            .message
            .as_ref()
            .unwrap()
            .content
    );

    println!("✅ All operations completed successfully!");

    Ok(())
}
