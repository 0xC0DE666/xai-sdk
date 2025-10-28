use crate::{XAI_API_URL, models_client::ModelsClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new ModelsClient connected to the xAI API.
///
/// # Returns
/// * `Result<ModelsClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = models::create_client().await?;
/// ```
pub async fn create_client() -> Result<ModelsClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(ModelsClient::new(channel))
}
