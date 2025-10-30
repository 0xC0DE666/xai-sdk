pub mod client {
    use crate::auth_client::AuthClient;
    use crate::common;
    use tonic::service::Interceptor;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    /// Creates a new AuthClient connected to the xAI API.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    /// # Example
    /// ```rust
    /// let client = auth::client::new("your-api-key-here").await?;
    /// ```
    pub async fn new(
        api_key: &str,
    ) -> Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = AuthClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new AuthClient with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    /// # Example
    /// ```rust
    /// let channel = common::channel::new().await?;
    /// let client = auth::client::with_channel(channel, "your-api-key-here")?;
    /// ```
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = AuthClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new `AuthClient` using a provided interceptor.
    ///
    /// Uses the same channel setup as [`new()`] but applies the custom interceptor.
    ///
    /// # Arguments
    /// * `interceptor` - Custom interceptor for request authentication/metadata
    ///
    /// # Returns
    /// * `Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>`
    ///   - The connected, intercepted client or a connection error
    ///
    /// # Example
    /// ```rust
    /// use xai_sdk::auth;
    /// use tonic::service::Interceptor;
    ///
    /// let custom = |req: tonic::Request<()>| -> Result<_, tonic::Status> { Ok(req) };
    /// let client = auth::client::with_interceptor(custom).await?;
    /// ```
    pub async fn with_interceptor(
        interceptor: impl Interceptor,
    ) -> Result<AuthClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let client = AuthClient::with_interceptor(channel, interceptor);

        Ok(client)
    }
}
