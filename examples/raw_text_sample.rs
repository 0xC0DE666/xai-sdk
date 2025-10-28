use anyhow::{Context, Result};
use std::env;
use tonic::{
    metadata::MetadataValue,
    Request,
    transport::Channel,
};
use xai_sdk::sample_client::SampleClient;
use xai_sdk::{SampleTextRequest, SampleTextResponse};

/// Creates a gRPC client
async fn create_client() -> Result<SampleClient<Channel>> {
    // Create the gRPC channel - try different endpoint
    let channel = Channel::from_static("https://api.x.ai:443")
        .connect()
        .await
        .context("Failed to connect to xAI API")?;

    // Create the client
    let client = SampleClient::new(channel);

    Ok(client)
}

/// Demonstrates raw text sampling with various parameters
async fn demonstrate_text_sampling() -> Result<()> {
    println!("🚀 xAI Raw Text Sampling Demo");
    println!("=============================");
    println!();

    // Load API key for authentication
    let api_key = env::var("XAI_API_KEY")
        .context("XAI_API_KEY environment variable must be set")?;

    // Create client
    let mut client = create_client().await?;

    // Example 1: Creative writing
    println!("📝 Example 1: Creative Writing");
    println!("-------------------------------");
    let request1 = Request::new(SampleTextRequest {
        prompt: vec!["Write a short poem about Rust programming language.".to_string()],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(200),
        temperature: Some(0.8),
        top_p: Some(0.9),
        n: Some(1),
        logprobs: true,
        top_logprobs: Some(3),
        user: "rust-demo".to_string(),
        ..Default::default()
    });

    call_sample_api(&mut client, request1, "Creative poem about Rust", &api_key).await?;

    println!();

    // Example 2: Technical explanation
    println!("🔧 Example 2: Technical Explanation");
    println!("-----------------------------------");
    let request2 = Request::new(SampleTextRequest {
        prompt: vec!["Explain the concept of ownership in Rust programming language in simple terms.".to_string()],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(300),
        temperature: Some(0.3),
        top_p: Some(0.8),
        n: Some(1),
        logprobs: false,
        user: "rust-demo".to_string(),
        ..Default::default()
    });

    call_sample_api(&mut client, request2, "Technical explanation of Rust ownership", &api_key).await?;

    println!();

    // Example 3: Multiple completions
    println!("🎯 Example 3: Multiple Completions");
    println!("----------------------------------");
    let request3 = Request::new(SampleTextRequest {
        prompt: vec!["Complete this sentence: 'The best programming language for systems programming is'".to_string()],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: Some(0.9),
        n: Some(3),
        logprobs: false,
        user: "rust-demo".to_string(),
        ..Default::default()
    });

    call_sample_api(&mut client, request3, "Multiple completions", &api_key).await?;

    Ok(())
}

/// Helper function to make API calls and handle responses
async fn call_sample_api(
    client: &mut SampleClient<Channel>,
    mut request: Request<SampleTextRequest>,
    description: &str,
    api_key: &str,
) -> Result<()> {
    // Add authentication header
    let token = MetadataValue::try_from(format!("Bearer {}", api_key))
        .context("Invalid API key format")?;
    request.metadata_mut().insert("authorization", token);

    println!("📤 Sending request: {}", description);
    println!("🤖 Model: {}", request.get_ref().model);
    println!("⚙️  Max tokens: {:?}, Temperature: {:?}, Top-p: {:?}", 
             request.get_ref().max_tokens, 
             request.get_ref().temperature, 
             request.get_ref().top_p);
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
                println!("   Text: {}", choice.text);
                println!("   Finish reason: {:?}", choice.finish_reason);
                println!();
            }
        }
        Err(e) => {
            eprintln!("❌ Error calling xAI API: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Run the demonstration
    demonstrate_text_sampling().await?;

    println!("🎉 Demo completed successfully!");
    Ok(())
}
