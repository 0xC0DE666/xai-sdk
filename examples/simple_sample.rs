use anyhow::{Context, Result};
use std::env;
use tonic::{metadata::MetadataValue, Request, transport::Channel};
use grok_grpc::xai_api::sample_client::SampleClient;
use grok_grpc::xai_api::{SampleTextRequest, SampleTextResponse};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key = env::var("XAI_API_KEY")
        .context("XAI_API_KEY environment variable must be set")?;

    // Create the gRPC channel - try different endpoint
    let channel = Channel::from_static("https://api.x.ai:443")
        .connect()
        .await
        .context("Failed to connect to xAI API")?;

    // Create the client
    let mut client = SampleClient::new(channel);

    // Create the request
    let mut request = Request::new(SampleTextRequest {
        prompt: vec!["Write a haiku about programming.".to_string()],
        model: "grok-3-latest".to_string(),
        max_tokens: None,
        temperature: Some(0.7),
        top_p: Some(0.9),
        n: Some(1),
        logprobs: false,
        user: "simple-demo".to_string(),
        ..Default::default()
    });

    // Add authentication header
    let token = MetadataValue::try_from(format!("Bearer {}", api_key))
        .context("Invalid API key format")?;
    request.metadata_mut().insert("authorization", token);

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: Write a haiku about programming.");
    println!("🤖 Model: grok-2-latest");
    println!();

    // Make the API call
    match client.sample_text(request).await {
        Ok(response) => {
            let sample_response: SampleTextResponse = response.into_inner();
            
            println!("✅ Response received!");
            println!("🆔 Request ID: {}", sample_response.id);
            println!("🤖 Model used: {}", sample_response.model);
            println!("📊 Usage: {:?}", sample_response.usage);
            println!();

            // Display the generated text
            for (i, choice) in sample_response.choices.iter().enumerate() {
                println!("📄 Choice {}:", i + 1);
                println!("   {}", choice.text);
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
