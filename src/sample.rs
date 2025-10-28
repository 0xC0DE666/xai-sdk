use crate::{XAI_API_URL, sample_client::SampleClient};
use tonic::transport::{Channel, ClientTlsConfig};

/// Creates a new SampleClient connected to the xAI API.
///
/// # Returns
/// * `Result<SampleClient<Channel>, tonic::transport::Error>` - The connected client or connection error
///
/// # Example
/// ```rust
/// let client = sample::create_client().await?;
/// ```
pub async fn create_client() -> Result<SampleClient<Channel>, tonic::transport::Error> {
    let channel = Channel::from_static(XAI_API_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    Ok(SampleClient::new(channel))
}
