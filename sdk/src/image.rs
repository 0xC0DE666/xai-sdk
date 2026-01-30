//! Image generation service client.
//!
//! Provides gRPC clients for generating high-quality images from text prompts
//! using xAI's advanced image generation models.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::image_client::ImageClient as XImageClient;

    pub type ImageClient = XImageClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `ImageClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection with Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ImageClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<ImageClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XImageClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `ImageClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `ImageClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> ImageClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XImageClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ImageClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<ImageClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<ImageClient, Error> {
        let channel = common::channel::new().await?;
        let client = XImageClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `ImageClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `ImageClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> ImageClient {
        XImageClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
