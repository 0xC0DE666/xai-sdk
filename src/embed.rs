//! Embedding service client.
//!
//! Provides clients for generating embeddings from text or images.

pub mod client {
    use crate::common;
    use crate::embedder_client::EmbedderClient;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new EmbedderClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<EmbedderClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<
        EmbedderClient<InterceptedService<Channel, impl Interceptor>>,
        tonic::transport::Error,
    > {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = EmbedderClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `EmbedderClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `EmbedderClient<InterceptedService<Channel, impl Interceptor>>` - The connected client
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> EmbedderClient<InterceptedService<Channel, impl Interceptor>> {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = EmbedderClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `EmbedderClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<EmbedderClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<
        EmbedderClient<InterceptedService<Channel, impl Interceptor>>,
        tonic::transport::Error,
    > {
        let channel = common::channel::new().await?;
        let client = EmbedderClient::with_interceptor(channel, interceptor);
        Ok(client)
    }

    /// Creates a new `EmbedderClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `EmbedderClient<InterceptedService<Channel, impl Interceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor,
    ) -> EmbedderClient<InterceptedService<Channel, impl Interceptor>> {
        EmbedderClient::with_interceptor(channel, interceptor)
    }
}
