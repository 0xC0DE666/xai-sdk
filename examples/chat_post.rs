use anyhow::{Context, Result};
use std::env;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, metadata::MetadataValue};

use xai_grpc::chat_client::ChatClient;
use grok_grpc::{
    Content, GetChatCompletionResponse, GetCompletionsRequest, Message, MessageRole, content,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create the gRPC channel - try different endpoint
    let channel = Channel::from_static("https://api.x.ai:443")
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await
        .context("Failed to connect to xAI API")?;

    // Create the client
    let mut client = ChatClient::new(channel);

    // Create the request
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text("Quote Hannibal Lectre.".into()));
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
                println!("📄 Choice {}:", i + 1);
                println!("   {}", choice.message.clone().unwrap().content);
                println!("   Finish reason: {:?}", choice.finish_reason);
            }
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
