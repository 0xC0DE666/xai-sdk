use anyhow::{Context, Result};
use std::env;
use tonic::Request;
use xai_sdk::sample;
use xai_sdk::xai_api::{SampleTextRequest, SampleTextResponse};

/// Demonstrates raw text sampling with various parameters
async fn demonstrate_text_sampling() -> Result<()> {
    println!("🚀 xAI Raw Text Sampling Demo");
    println!("=============================");
    println!();

    // Load API key for authentication
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated client
    let mut client = sample::client::new(&api_key).await?;

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

    // Example 1: Creative writing
    println!("📤 Sending request: Creative poem about Rust");
    println!("🤖 Model: {}", request1.get_ref().model);
    println!(
        "⚙️  Max tokens: {:?}, Temperature: {:?}, Top-p: {:?}",
        request1.get_ref().max_tokens,
        request1.get_ref().temperature,
        request1.get_ref().top_p
    );
    println!();

    match client.sample_text(request1).await {
        Ok(response) => {
            let sample_response: SampleTextResponse = response.into_inner();
            println!("✅ Response received!");
            println!("🆔 Request ID: {}", sample_response.id);
            println!("🤖 Model used: {}", sample_response.model);
            println!("📊 Usage: {:?}", sample_response.usage);
            println!();

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

    println!();

    // Example 2: Technical explanation
    println!("🔧 Example 2: Technical Explanation");
    println!("-----------------------------------");
    let request2 = Request::new(SampleTextRequest {
        prompt: vec![
            "Explain the concept of ownership in Rust programming language in simple terms."
                .to_string(),
        ],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(300),
        temperature: Some(0.3),
        top_p: Some(0.8),
        n: Some(1),
        logprobs: false,
        user: "rust-demo".to_string(),
        ..Default::default()
    });

    println!("📤 Sending request: Technical explanation of Rust ownership");
    println!("🤖 Model: {}", request2.get_ref().model);
    println!(
        "⚙️  Max tokens: {:?}, Temperature: {:?}, Top-p: {:?}",
        request2.get_ref().max_tokens,
        request2.get_ref().temperature,
        request2.get_ref().top_p
    );
    println!();

    match client.sample_text(request2).await {
        Ok(response) => {
            let sample_response: SampleTextResponse = response.into_inner();
            println!("✅ Response received!");
            println!("🆔 Request ID: {}", sample_response.id);
            println!("🤖 Model used: {}", sample_response.model);
            println!("📊 Usage: {:?}", sample_response.usage);
            println!();

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

    println!();

    // Example 3: Multiple completions
    println!("🎯 Example 3: Multiple Completions");
    println!("----------------------------------");
    let request3 = Request::new(SampleTextRequest {
        prompt: vec![
            "Complete this sentence: 'The best programming language for systems programming is'"
                .to_string(),
        ],
        model: "grok-2-latest".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: Some(0.9),
        n: Some(3),
        logprobs: false,
        user: "rust-demo".to_string(),
        ..Default::default()
    });

    println!("📤 Sending request: Multiple completions");
    println!("🤖 Model: {}", request3.get_ref().model);
    println!(
        "⚙️  Max tokens: {:?}, Temperature: {:?}, Top-p: {:?}",
        request3.get_ref().max_tokens,
        request3.get_ref().temperature,
        request3.get_ref().top_p
    );
    println!();

    match client.sample_text(request3).await {
        Ok(response) => {
            let sample_response: SampleTextResponse = response.into_inner();
            println!("✅ Response received!");
            println!("🆔 Request ID: {}", sample_response.id);
            println!("🤖 Model used: {}", sample_response.model);
            println!("📊 Usage: {:?}", sample_response.usage);
            println!();

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
