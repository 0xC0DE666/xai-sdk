//! Image generation service client.
//!
//! Provides clients for generating images from text prompts.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::image_client::ImageClient as XImageClient;

    pub type ImageClient = XImageClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new ImageClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ImageClient, Error>` - The connected client or connection error
    ///
    pub async fn new(api_key: &str) -> Result<ImageClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XImageClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `ImageClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `ImageClient` - The connected client
    pub fn with_channel(channel: Channel, api_key: &str) -> ImageClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XImageClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ImageClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<ImageClient, Error>` - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<ImageClient, Error> {
        let channel = common::channel::new().await?;
        let client = XImageClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `ImageClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `ImageClient` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> ImageClient {
        XImageClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
