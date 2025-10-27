use anyhow::Result;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing xAI API connection...");
    
    // Test different endpoints
    let endpoints = [
        "https://api.x.ai",
        "https://api.x.ai:443", 
        "https://api.x.ai:80",
        "http://api.x.ai",
        "http://api.x.ai:80",
    ];
    
    for endpoint in &endpoints {
        println!("Testing endpoint: {}", endpoint);
        match Channel::from_static(endpoint).connect().await {
            Ok(_channel) => {
                println!("✅ Successfully connected to {}", endpoint);
            }
            Err(e) => {
                println!("❌ Failed to connect to {}: {}", endpoint, e);
            }
        }
        println!();
    }
    
    Ok(())
}
