use crate::{XAI_API_URL, documents_client::DocumentsClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new DocumentsClient connected to the xAI API.
///
/// # Returns
/// * `Result<DocumentsClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = documents::create_client().await?;
/// ```
pub async fn create_client() -> Result<DocumentsClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(DocumentsClient::new(channel))
}
