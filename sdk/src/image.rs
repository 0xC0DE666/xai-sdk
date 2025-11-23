//! Image generation service client.
//!
//! Provides clients for generating images from text prompts.

pub mod client {
    use crate::common;
    use crate::xai_api::image_client::ImageClient;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new ImageClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ImageClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `ImageClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `ImageClient<InterceptedService<Channel, impl Interceptor>>` - The connected client
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> ImageClient<InterceptedService<Channel, impl Interceptor>> {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ImageClient::with_interceptor(channel, auth_intercept);

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
    /// * `Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let client = ImageClient::with_interceptor(channel, interceptor);
        Ok(client)
    }

    /// Creates a new `ImageClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `ImageClient<InterceptedService<Channel, impl Interceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor,
    ) -> ImageClient<InterceptedService<Channel, impl Interceptor>> {
        ImageClient::with_interceptor(channel, interceptor)
    }
}
