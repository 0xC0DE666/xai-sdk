//! Text sampling service client.
//!
//! Provides clients for raw text generation and sampling operations.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::xai_api::sample_client::SampleClient as XSampleClient;

    pub type SampleClient = XSampleClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new SampleClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<SampleClient, Error>` - The connected client or connection error
    ///
    pub async fn new(api_key: &str) -> Result<SampleClient, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XSampleClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `SampleClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `SampleClient` - The connected client
    pub fn with_channel(channel: Channel, api_key: &str) -> SampleClient {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = XSampleClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `SampleClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<SampleClient, Error>` - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<SampleClient, Error> {
        let channel = common::channel::new().await?;
        let client = XSampleClient::with_interceptor(channel, ClientInterceptor::new(interceptor));
        Ok(client)
    }

    /// Creates a new `SampleClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `SampleClient` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> SampleClient {
        XSampleClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
