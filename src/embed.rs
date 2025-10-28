use crate::{XAI_API_URL, embedder_client::EmbedderClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new EmbedderClient connected to the xAI API.
///
/// # Returns
/// * `Result<EmbedderClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = embed::create_client().await?;
/// ```
pub async fn create_client() -> Result<EmbedderClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(EmbedderClient::new(channel))
}
