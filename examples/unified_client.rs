use xai_sdk::{client::XaiClient, SampleTextRequest, GetCompletionsRequest, Message, MessageRole, Content, content};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = std::env::var("XAI_API_KEY")
        .unwrap_or_else(|_| "your-api-key-here".to_string());

    // Create unified client with authentication
    let mut client = XaiClient::with_auth(&api_key).await?;
    
    println!("🚀 xAI SDK Unified Client Example");
    println!("================================\n");

    // List available models
    println!("📋 Listing available models...");
    let models_request = client.models_request().await?;
    let models_response = client.models.list_language_models(models_request).await?;
    println!("Available models: {:?}\n", models_response.into_inner().models);

    // Generate text using sample service
    println!("✍️  Generating text...");
    let sample_request = SampleTextRequest {
        prompt: vec!["Write a haiku about Rust programming".to_string()],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.8),
        ..Default::default()
    };
    let authenticated_sample_request = client.sample_request(sample_request).await?;
    let sample_response = client.sample.sample_text(authenticated_sample_request).await?;
    println!("Generated: {}\n", sample_response.into_inner().choices[0].text);

    // Chat completion
    println!("💬 Chat completion...");
    let message = Message {
        role: MessageRole::RoleUser.into(),
        content: vec![Content {
            content: Some(content::Content::Text("Explain Rust ownership in simple terms".to_string())),
        }],
        ..Default::default()
    };
    let chat_request = GetCompletionsRequest {
        model: "grok-3-latest".to_string(),
        messages: vec![message],
        ..Default::default()
    };
    let authenticated_chat_request = client.chat_request(chat_request).await?;
    let chat_response = client.chat.get_completion(authenticated_chat_request).await?;
    println!("Chat response: {}\n", chat_response.into_inner().choices[0].message.as_ref().unwrap().content);

    println!("✅ All operations completed successfully!");

    Ok(())
}
