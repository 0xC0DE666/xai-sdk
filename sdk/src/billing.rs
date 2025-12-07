//! Billing service client.
//!
//! Provides clients for managing billing information, payment methods, invoices,
//! prepaid credits, and spending limits.

pub mod client {
    use crate::common;
    use crate::common::interceptor::ClientInterceptor;
    use crate::export::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::transport::{Channel, Error};
    use crate::prod_mc_billing::ui_svc_client::UiSvcClient;

    /// Creates a new `UiSvcClient` connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<UiSvcClient<InterceptedService<Channel, ClientInterceptor>>, Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<UiSvcClient<InterceptedService<Channel, ClientInterceptor>>, Error> {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = UiSvcClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `UiSvcClient` with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `UiSvcClient<InterceptedService<Channel, ClientInterceptor>>` - The connected client
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> UiSvcClient<InterceptedService<Channel, ClientInterceptor>> {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = UiSvcClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `UiSvcClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<UiSvcClient<InterceptedService<Channel, ClientInterceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<UiSvcClient<InterceptedService<Channel, ClientInterceptor>>, Error> {
        let channel = common::channel::new().await?;
        let client = UiSvcClient::with_interceptor(channel, ClientInterceptor::new(interceptor));

        Ok(client)
    }

    /// Creates a new `UiSvcClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `UiSvcClient<InterceptedService<Channel, ClientInterceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> UiSvcClient<InterceptedService<Channel, ClientInterceptor>> {
        UiSvcClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
