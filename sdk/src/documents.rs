//! Document processing service client.
//!
//! Provides gRPC clients for searching and processing document collections,
//! enabling semantic document search and retrieval capabilities.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::documents_client::DocumentsClient as XDocumentsClient;

    pub type DocumentsClient = XDocumentsClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `DocumentsClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection with Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<DocumentsClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<DocumentsClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XDocumentsClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `DocumentsClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `DocumentsClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> DocumentsClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XDocumentsClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `DocumentsClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<DocumentsClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<DocumentsClient, Error> {
        let channel = common::channel::new().await?;
        let client = XDocumentsClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `DocumentsClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `DocumentsClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> DocumentsClient {
        XDocumentsClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
