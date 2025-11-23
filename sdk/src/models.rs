//! Model information service client.
//!
//! Provides clients for listing available language models, embedding models, and image generation models.

pub mod client {
    use crate::common;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::models_client::ModelsClient;

    /// Creates a new ModelsClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ModelsClient<InterceptedService<Channel, impl Interceptor>>, Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<ModelsClient<InterceptedService<Channel, impl Interceptor>>, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ModelsClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `ModelsClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `ModelsClient<InterceptedService<Channel, impl Interceptor>>` - The connected client
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> ModelsClient<InterceptedService<Channel, impl Interceptor>> {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ModelsClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ModelsClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<ModelsClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<ModelsClient<InterceptedService<Channel, impl Interceptor>>, Error> {
        let channel = common::channel::new().await?;
        let client = ModelsClient::with_interceptor(channel, interceptor);
        Ok(client)
    }

    /// Creates a new `ModelsClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `ModelsClient<InterceptedService<Channel, impl Interceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor,
    ) -> ModelsClient<InterceptedService<Channel, impl Interceptor>> {
        ModelsClient::with_interceptor(channel, interceptor)
    }
}
