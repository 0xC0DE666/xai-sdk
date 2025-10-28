use crate::{XAI_API_URL, image_client::ImageClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new ImageClient connected to the xAI API.
///
/// # Returns
/// * `Result<ImageClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = image::create_client().await?;
/// ```
pub async fn create_client() -> Result<ImageClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(ImageClient::new(channel))
}
