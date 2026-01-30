//! Authentication service client.
//!
//! Provides gRPC clients for API key validation, metadata retrieval, and authentication operations.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::auth_client::AuthClient as XAuthClient;

    pub type AuthClient = XAuthClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `AuthClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection with Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<AuthClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<AuthClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XAuthClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `AuthClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `AuthClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> AuthClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XAuthClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `AuthClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<AuthClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<AuthClient, Error> {
        let channel = common::channel::new().await?;
        let client = XAuthClient::with_interceptor(channel, ClientInterceptor::new(interceptor));

        Ok(client)
    }

    /// Creates a new `AuthClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `AuthClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> AuthClient {
        XAuthClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
