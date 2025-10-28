use anyhow::Result;
use tonic::transport::{Channel, ClientTlsConfig};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing xAI API connection...");

    // Test different endpoints
    let endpoints = [
        "https://api.x.ai",
        "https://api.x.ai:443",
        "http://api.x.ai",
        "http://api.x.ai:80",
    ];

    for endpoint in &endpoints {
        println!("Testing endpoint: {}", endpoint);
        // Create the gRPC channel - try different endpoint
        let channel = Channel::from_static(endpoint)
            .tls_config(ClientTlsConfig::new().with_native_roots())?;

        match channel.connect().await {
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
