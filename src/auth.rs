use crate::{XAI_API_URL, auth_client::AuthClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new AuthClient connected to the xAI API.
///
/// # Returns
/// * `Result<AuthClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = auth::create_client().await?;
/// ```
pub async fn create_client() -> Result<AuthClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(AuthClient::new(channel))
}
