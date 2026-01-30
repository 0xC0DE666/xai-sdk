//! Video generation service client.
//!
//! Provides gRPC clients for generating high-quality videos using xAI's
//! advanced video generation models with support for deferred processing.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::video_client::VideoClient as XVideoClient;

    pub type VideoClient = XVideoClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `VideoClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection with Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<VideoClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<VideoClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XVideoClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `VideoClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `VideoClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> VideoClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XVideoClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `VideoClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<VideoClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<VideoClient, Error> {
        let channel = common::channel::new().await?;
        let client = XVideoClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `VideoClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `VideoClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> VideoClient {
        XVideoClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}