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
    /// # Example
    /// ```rust
    /// let client = tokenize::client::new("your-api-key-here").await?;
    /// ```
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
    /// # Example
    /// ```rust
    /// let channel = common::channel::new().await?;
    /// let client = tokenize::client::with_channel(channel, "your-api-key-here")?;
    /// ```
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> Result<TokenizeClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = TokenizeClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }
}
