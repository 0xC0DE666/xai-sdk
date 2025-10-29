use anyhow::Result;
use xai_sdk::common;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing xAI API connection...");

    // Test connection using the common channel creation
    match common::channel::new().await {
        Ok(_channel) => {
            println!("✅ Successfully connected to xAI API");
            println!("🌐 Endpoint: https://api.x.ai:443");
        }
        Err(e) => {
            println!("❌ Failed to connect to xAI API: {}", e);
            return Err(e.into());
        }
    }

    println!();
    println!("🎉 Connection test completed successfully!");

    Ok(())
}
