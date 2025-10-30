pub mod client {
    use crate::common;
    use crate::tokenize_client::TokenizeClient;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new TokenizeClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    pub async fn new(
        api_key: &str,
    ) -> Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = TokenizeClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new TokenizeClient with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = TokenizeClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `TokenizeClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let client = TokenizeClient::with_interceptor(channel, interceptor);
        Ok(client)
    }

    /// Creates a new `TokenizeClient` with an existing channel and a provided interceptor.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `TokenizeClient<InterceptedService<Channel, impl Interceptor>>` - The intercepted client
    pub fn with_channel_and_interceptor(
        channel: Channel,
        interceptor: impl Interceptor,
    ) -> TokenizeClient<InterceptedService<Channel, impl Interceptor>> {
        TokenizeClient::with_interceptor(channel, interceptor)
    }
}
