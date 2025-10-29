use anyhow::{Context, Result};
use std::env;
use tonic::Request;

use xai_sdk::{
    chat, Content, GetChatCompletionResponse, GetCompletionsRequest, Message, MessageRole, content,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variable
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated chat client
    let mut client = chat::client::new(&api_key).await?;

    // Create the request
    let mut cntnt = Content::default();
    cntnt.content = Some(content::Content::Text("Quote Hannibal Lectre.".into()));
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

    println!("🚀 Sending request to xAI API...");
    println!("📝 Prompt: Quote Hannibal Lectre.");
    println!("🤖 Model: grok-3-latest");
    println!();

    // Make the API call - authentication is automatic!
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
