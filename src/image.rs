pub mod client {
    use crate::common;
    use crate::image_client::ImageClient;
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
    /// # Example
    /// ```rust
    /// let client = image::client::new("your-api-key-here").await?;
    /// ```
    pub async fn new(
        api_key: &str,
    ) -> Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let channel = common::channel::new().await?;
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ImageClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }

    /// Creates a new ImageClient with an existing channel.
    ///
    /// # Arguments
    /// * `channel` - An existing gRPC channel
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>` - The connected client or connection error
    ///
    /// # Example
    /// ```rust
    /// let channel = common::channel::new().await?;
    /// let client = image::client::with_channel(channel, "your-api-key-here")?;
    /// ```
    pub fn with_channel(
        channel: Channel,
        api_key: &str,
    ) -> Result<ImageClient<InterceptedService<Channel, impl Interceptor>>, tonic::transport::Error>
    {
        let auth_intercept = common::interceptor::auth(api_key);
        let client = ImageClient::with_interceptor(channel, auth_intercept);

        Ok(client)
    }
}
