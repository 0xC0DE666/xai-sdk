use crate::{
    XAI_API_URL, auth_client::AuthClient, chat_client::ChatClient,
    documents_client::DocumentsClient, embedder_client::EmbedderClient, image_client::ImageClient,
    models_client::ModelsClient, sample_client::SampleClient, tokenize_client::TokenizeClient,
};
use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig};
use anyhow::Result;

/// Unified xAI client that provides access to all xAI services.
///
/// This client wraps all individual service clients and provides a convenient
/// interface for interacting with the xAI API. It handles authentication
/// automatically for all requests.
///
/// # Example
/// ```rust
/// use xai_sdk::XaiClient;
/// use tonic::Request;
///
/// let client = XaiClient::with_auth("your-api-key-here").await?;
/// let response = client.chat.get_completion(request).await?;
/// ```
pub struct XaiClient {
    /// API key for authentication
    api_key: String,
    /// Authentication service client
    pub auth: AuthClient<Channel>,
    /// Chat completions service client
    pub chat: ChatClient<Channel>,
    /// Document processing service client
    pub documents: DocumentsClient<Channel>,
    /// Embeddings service client
    pub embed: EmbedderClient<Channel>,
    /// Image generation service client
    pub image: ImageClient<Channel>,
    /// Model listing service client
    pub models: ModelsClient<Channel>,
    /// Text sampling service client
    pub sample: SampleClient<Channel>,
    /// Tokenization service client
    pub tokenize: TokenizeClient<Channel>,
}

impl XaiClient {
    /// Creates a new XaiClient with authentication for all requests.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key for authentication
    ///
    /// # Returns
    /// * `Result<XaiClient, tonic::transport::Error>` - The authenticated client or connection error
    ///
    /// # Example
    /// ```rust
    /// let client = XaiClient::with_auth("your-api-key-here").await?;
    /// ```
    pub async fn with_auth(api_key: &str) -> Result<Self, tonic::transport::Error> {
        let channel = Self::create_channel().await?;

        // Create all clients
        let auth = AuthClient::new(channel.clone());
        let chat = ChatClient::new(channel.clone());
        let documents = DocumentsClient::new(channel.clone());
        let embed = EmbedderClient::new(channel.clone());
        let image = ImageClient::new(channel.clone());
        let models = ModelsClient::new(channel.clone());
        let sample = SampleClient::new(channel.clone());
        let tokenize = TokenizeClient::new(channel);

        Ok(Self {
            api_key: api_key.to_string(),
            auth,
            chat,
            documents,
            embed,
            image,
            models,
            sample,
            tokenize,
        })
    }

    /// Creates a new XaiClient without authentication.
    ///
    /// # Returns
    /// * `Result<XaiClient, tonic::transport::Error>` - The unauthenticated client or connection error
    ///
    /// # Example
    /// ```rust
    /// let client = XaiClient::new().await?;
    /// // You'll need to add authentication to individual requests
    /// ```
    pub async fn new() -> Result<Self, tonic::transport::Error> {
        Self::with_auth("").await
    }

    /// Adds authentication header to a request using the stored API key.
    ///
    /// # Arguments
    /// * `request` - The request to add authentication to
    ///
    /// # Returns
    /// * `Result<Request<T>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request or error
    ///
    /// # Example
    /// ```rust
    /// let request = Request::new(GetCompletionsRequest::default());
    /// let authenticated_request = client.add_auth(request).await?;
    /// ```
    pub async fn add_auth<T>(&self, mut request: Request<T>) -> Result<Request<T>> {
        if !self.api_key.is_empty() {
            let token = MetadataValue::try_from(format!("Bearer {}", self.api_key))?;
            request.metadata_mut().insert("authorization", token);
        }
        Ok(request)
    }

    /// Creates a gRPC channel to the xAI API.
    ///
    /// # Returns
    /// * `Result<Channel, tonic::transport::Error>` - The channel or connection error
    fn create_channel()
    -> impl std::future::Future<Output = Result<Channel, tonic::transport::Error>> {
        async {
            Channel::from_static(XAI_API_URL)
                .tls_config(ClientTlsConfig::new().with_native_roots())?
                .connect()
                .await
        }
    }
}

/// Convenience methods for common operations
impl XaiClient {
    /// Creates a chat completion request with authentication.
    ///
    /// # Arguments
    /// * `request` - The chat completion request
    ///
    /// # Returns
    /// * `Result<Request<GetCompletionsRequest>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request
    pub async fn chat_request(
        &self,
        request: crate::GetCompletionsRequest,
    ) -> Result<Request<crate::GetCompletionsRequest>>
    {
        self.add_auth(Request::new(request)).await
    }

    /// Creates a text sampling request with authentication.
    ///
    /// # Arguments
    /// * `request` - The text sampling request
    ///
    /// # Returns
    /// * `Result<Request<SampleTextRequest>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request
    pub async fn sample_request(
        &self,
        request: crate::SampleTextRequest,
    ) -> Result<Request<crate::SampleTextRequest>> {
        self.add_auth(Request::new(request)).await
    }

    /// Creates a model listing request with authentication.
    ///
    /// # Returns
    /// * `Result<Request<()>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request
    pub async fn models_request(
        &self,
    ) -> Result<Request<()>> {
        self.add_auth(Request::new(())).await
    }

    /// Creates an embedding request with authentication.
    ///
    /// # Arguments
    /// * `request` - The embedding request
    ///
    /// # Returns
    /// * `Result<Request<EmbedRequest>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request
    pub async fn embed_request(
        &self,
        request: crate::EmbedRequest,
    ) -> Result<Request<crate::EmbedRequest>> {
        self.add_auth(Request::new(request)).await
    }

    /// Creates an image generation request with authentication.
    ///
    /// # Arguments
    /// * `request` - The image generation request
    ///
    /// # Returns
    /// * `Result<Request<GenerateImageRequest>, Box<dyn std::error::Error + Send + Sync>>` - The authenticated request
    pub async fn image_request(
        &self,
        request: crate::GenerateImageRequest,
    ) -> Result<Request<crate::GenerateImageRequest>>
    {
        self.add_auth(Request::new(request)).await
    }
}
