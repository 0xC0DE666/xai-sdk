pub mod channel {
    use crate::XAI_API_URL;
    use tonic::transport::{Channel, ClientTlsConfig};

    /// Create a TLS-enabled gRPC `Channel` to the xAI API endpoint.
    ///
    /// This configures Tonic with native root certificates and connects to
    /// the SDK's default endpoint defined by [`XAI_API_URL`].
    ///
    /// # Returns
    /// * `Result<Channel, tonic::transport::Error>` - A connected channel or a connection error
    ///
    pub async fn new() -> Result<Channel, tonic::transport::Error> {
        Channel::from_static(XAI_API_URL)
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await
    }
}

pub mod interceptor {
    use tonic::Request;
    use tonic::Status;
    use tonic::metadata::MetadataValue;
    use tonic::service::Interceptor;

    /// Build an interceptor that injects a Bearer token `authorization` header.
    ///
    /// The header value is set to `Bearer {api_key}`. This is the default
    /// authentication mechanism used by the SDK's `client::new(api_key)` helpers.
    ///
    /// # Arguments
    /// * `api_key` - The xAI API key used for Bearer authentication
    ///
    /// # Returns
    /// * `impl Interceptor` - An interceptor that adds the authorization metadata
    ///
    pub fn auth(api_key: &str) -> impl Interceptor {
        move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let token = MetadataValue::try_from(&format!("Bearer {}", api_key)).map_err(|e| {
                Status::invalid_argument(format!("Failed to create metadata value: {}", e))
            })?;

            req.metadata_mut().insert("authorization", token);

            Ok(req)
        }
    }

    /// A boxed interceptor function type alias for composition.
    ///
    /// This function takes a `Request<()>` and returns either the (possibly modified)
    /// request or a `Status` error.
    pub type DynInterceptor = Box<
        dyn FnMut(Request<()>) -> Result<Request<()>, Status> + Send + Sync + 'static,
    >;

    /// Compose multiple interceptors into a single interceptor, applied in order.
    ///
    /// Each interceptor receives the output request of the previous one; the first
    /// receives the original request. If any interceptor returns an error, the
    /// composed interceptor returns that error immediately.
    ///
    /// # Arguments
    /// * `interceptors` - A vector of boxed interceptor functions applied sequentially
    ///
    /// # Returns
    /// * `impl Interceptor` - A single interceptor that applies all provided interceptors
    ///
    pub fn compose(mut interceptors: Vec<DynInterceptor>) -> impl Interceptor {
        move |mut req: Request<()>| -> Result<Request<()>, Status> {
            for f in interceptors.iter_mut() {
                req = f(req)?;
            }
            Ok(req)
        }
    }
}
