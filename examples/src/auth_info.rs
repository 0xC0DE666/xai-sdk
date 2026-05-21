use anyhow::{Context, Result};
use std::env;
use xai_sdk::export::tonic::Request;
use xai_sdk::api::ApiKey;
use xai_sdk::auth;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 xAI Auth Info Example");
    println!("========================\n");

    // Load API key for authentication
    let api_key =
        env::var("XAI_API_KEY").context("XAI_API_KEY environment variable must be set")?;

    // Create authenticated auth client
    let mut client = auth::client::new(&api_key).await?;

    // Build request (no body needed for get_api_key_info)
    let request = Request::new(());

    // Call the API
    match client.get_api_key_info(request).await {
        Ok(response) => {
            let key_info: ApiKey = response.into_inner();

            println!("✅ Auth info retrieved successfully\n");
            println!("🆔 API Key ID: {}", key_info.api_key_id);
            println!("🙍 User ID: {}", key_info.user_id);
            println!("👥 Team ID: {}", key_info.team_id);
            println!("🏷️  Name: {}", key_info.name);
            println!("🔒 Redacted Key: {}", key_info.redacted_api_key);
            println!("🚫 Key Blocked: {}", key_info.api_key_blocked);
            println!("🚫 Team Blocked: {}", key_info.team_blocked);
            println!("🛑 Disabled: {}", key_info.disabled);
            if let Some(created) = key_info.create_time {
                println!("🗓️  Created: {:?}", created);
            }
            if let Some(modified) = key_info.modify_time {
                println!("🗓️  Modified: {:?}", modified);
            }
            if !key_info.acls.is_empty() {
                println!("🔑 ACLs: {:?}", key_info.acls);
            }
        }
        Err(e) => {
            eprintln!("❌ Error fetching auth info: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
