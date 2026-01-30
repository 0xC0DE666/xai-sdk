//! Model information service client.
//!
//! Provides gRPC clients for querying available xAI models including language models,
//! embedding models, and image generation models with their capabilities and metadata.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::models_client::ModelsClient as XModelsClient;

    pub type ModelsClient = XModelsClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `ModelsClient` connected to the xAI API.
    ///
    /// Establishes a secure TLS connection with Bearer token authentication.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ModelsClient, Error>` - Connected client or transport error
    ///
    pub async fn new(api_key: &str) -> Result<ModelsClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XModelsClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `ModelsClient` using an existing gRPC channel.
    ///
    /// Useful for sharing connections across multiple service clients.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `api_key` - Valid xAI API key for authentication
    ///
    /// # Returns
    /// * `ModelsClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, api_key: &str) -> ModelsClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XModelsClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `ModelsClient` with a custom interceptor.
    ///
    /// Creates a new TLS connection but uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<ModelsClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<ModelsClient, Error> {
        let channel = common::channel::new().await?;
        let client = XModelsClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `ModelsClient` with an existing channel and custom interceptor.
    ///
    /// Combines a shared channel with custom request interception.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to xAI API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `ModelsClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> ModelsClient {
        XModelsClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
