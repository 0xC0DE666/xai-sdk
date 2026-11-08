//! Billing service client.
//!
//! Provides gRPC clients for managing billing information, payment methods, invoices,
//! prepaid credits, and spending limits.
//!
//! Billing lives on the **management** API (`management-api.x.ai`) and requires a
//! **management key** from the xAI console — inference API keys are rejected. Each
//! RPC is also scoped by `team_id`.

pub mod client {
    use crate::shared;
    use crate::shared::interceptor::ClientInterceptor;
    use crate::export::tonic::service::{Interceptor, interceptor::InterceptedService};
    use crate::export::tonic::transport::{Channel, Error};
    use crate::prod_mc_billing::ui_svc_client::UiSvcClient as XUiSvcClient;

    pub type BillingClient = XUiSvcClient<InterceptedService<Channel, ClientInterceptor>>;

    /// Creates a new authenticated `BillingClient` connected to the management API.
    ///
    /// Establishes a secure TLS connection to [`crate::XAI_MANAGEMENT_API_URL`] with
    /// Bearer token authentication.
    ///
    /// # Arguments
    /// * `management_key` - Management API key (not an inference key)
    ///
    /// # Returns
    /// * `Result<BillingClient, Error>` - Connected client or transport error
    ///
    pub async fn new(management_key: &str) -> Result<BillingClient, Error> {
        let channel = shared::channel::management().await?;
        let auth_intercept = shared::interceptor::auth(management_key);
        let client = XUiSvcClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new authenticated `BillingClient` using an existing gRPC channel.
    ///
    /// The channel must already target the management host
    /// ([`crate::XAI_MANAGEMENT_API_URL`] / [`shared::channel::management`]).
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to the management API
    /// * `management_key` - Management API key (not an inference key)
    ///
    /// # Returns
    /// * `BillingClient` - Authenticated client using the provided channel
    pub fn with_channel(channel: Channel, management_key: &str) -> BillingClient {
        let auth_intercept = shared::interceptor::auth(management_key);
        let client = XUiSvcClient::with_interceptor(channel, auth_intercept);

        client
    }

    /// Creates a new `BillingClient` with a custom interceptor.
    ///
    /// Opens a management-API channel and uses the provided interceptor instead of
    /// the default authentication interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `Result<BillingClient, Error>` - Intercepted client or connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> Result<BillingClient, Error> {
        let channel = shared::channel::management().await?;
        let client = XUiSvcClient::with_interceptor(channel, ClientInterceptor::new(interceptor));

        Ok(client)
    }

    /// Creates a new `BillingClient` with an existing channel and custom interceptor.
    ///
    /// The channel must already target the management host.
    ///
    /// # Arguments
    /// * `channel` - Existing TLS-secured gRPC channel to the management API
    /// * `interceptor` - Custom request interceptor (must handle authentication)
    ///
    /// # Returns
    /// * `BillingClient` - Intercepted client using the provided channel
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor + Send + Sync + 'static,
    ) -> BillingClient {
        XUiSvcClient::with_interceptor(channel, ClientInterceptor::new(interceptor))
    }
}
