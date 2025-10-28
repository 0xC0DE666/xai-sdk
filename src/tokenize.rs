use crate::{XAI_API_URL, tokenize_client::TokenizeClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new TokenizeClient connected to the xAI API.
///
/// # Returns
/// * `Result<TokenizeClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = tokenize::create_client().await?;
/// ```
pub async fn create_client() -> Result<TokenizeClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(TokenizeClient::new(channel))
}
