use crate::{XAI_API_URL, auth_client::AuthClient};
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::metadata::MetadataValue;
use tonic::Request;

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

/// Adds authentication header to a request.
///
/// # Arguments
/// * `request` - The request to add authentication to
/// * `api_key` - The xAI API key for authentication
///
/// # Returns
/// * `Result<Request<T>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request or error
///
/// # Example
/// ```rust
/// let mut request = Request::new(GetApiKeyInfoRequest::default());
/// let authenticated_request = auth::add_auth(request, "your-api-key-here")?;
/// ```
pub fn add_auth<T>(mut request: Request<T>, api_key: &str) -> Result<Request<T>, Box<dyn std::error::Error + Send + Sync>> {
    let token = MetadataValue::try_from(format!("Bearer {}", api_key))?;
    request.metadata_mut().insert("authorization", token);
    Ok(request)
}
