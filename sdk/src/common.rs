//! Common utilities for the xAI SDK.
//!
//! Provides shared functionality and utilities used across the xAI SDK,
//! including channel creation, authentication interceptors, and common types.

pub mod channel {
    use crate::XAI_API_URL;
    use crate::export::transport::{Channel, ClientTlsConfig, Error};

    /// Creates a TLS-enabled gRPC `Channel` to the xAI API endpoint.
    ///
    /// Configures Tonic with native root certificates and connects to
    /// the SDK's default endpoint defined by [`XAI_API_URL`].
    ///
    /// # Returns
    /// * `Result<Channel, Error>` - Connected channel or transport error
    ///
    pub async fn new() -> Result<Channel, Error> {
        Channel::from_static(XAI_API_URL)
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await
    }
}

pub mod interceptor {
    use crate::export::metadata::MetadataValue;
    use crate::export::service::Interceptor;
    use crate::export::{Request, Status};

    /// Concrete interceptor type for client contexts.
    ///
    /// Erases the concrete interceptor implementation, allowing use as a concrete type
    /// in return positions and stored in structs where `impl Interceptor` cannot be used.
    ///
    /// `Send + Sync`, making it safe to use across thread boundaries.
    pub struct ClientInterceptor {
        inner: Box<dyn Interceptor + Send + Sync>,
    }

    impl ClientInterceptor {
        /// Creates a new `ClientInterceptor` from any interceptor.
        ///
        /// The interceptor is boxed internally, allowing use as a concrete type
        /// in contexts where `impl Interceptor` cannot be used.
        ///
        /// # Arguments
        /// * `inner` - Any `Send + Sync` type implementing `Interceptor`
        ///
        pub fn new(inner: impl Interceptor + Send + Sync + 'static) -> Self {
            Self {
                inner: Box::new(inner),
            }
        }
    }

    impl From<Box<dyn Interceptor + Send + Sync>> for ClientInterceptor {
        fn from(inner: Box<dyn Interceptor + Send + Sync>) -> Self {
            Self { inner }
        }
    }

    impl Interceptor for ClientInterceptor {
        fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
            self.inner.call(request)
        }
    }

    /// Creates an interceptor that injects a Bearer token `authorization` header.
    ///
    /// Header value is set to `Bearer {api_key}`. This is the default
    /// authentication mechanism used by SDK client constructors.
    ///
    /// # Arguments
    /// * `api_key` - Valid xAI API key for Bearer authentication
    ///
    /// # Returns
    /// * `ClientInterceptor` - Interceptor that adds authorization metadata
    ///
    pub fn auth(api_key: &str) -> ClientInterceptor {
        let api_key = api_key.to_string();
        ClientInterceptor::new(move |mut req: Request<()>| -> Result<Request<()>, Status> {
            let token = MetadataValue::try_from(&format!("Bearer {}", api_key)).map_err(|e| {
                Status::invalid_argument(format!("Failed to create metadata value: {}", e))
            })?;

            req.metadata_mut().insert("authorization", token);

            Ok(req)
        })
    }

    /// Composes multiple interceptors into a single interceptor, applied in order.
    ///
    /// Each interceptor receives the output request of the previous one. If any interceptor
    /// returns an error, the composed interceptor returns that error immediately.
    ///
    /// # Arguments
    /// * `interceptors` - Vector of boxed interceptor functions applied sequentially
    ///
    /// # Returns
    /// * `ClientInterceptor` - Single interceptor that applies all provided interceptors
    ///
    pub fn compose(mut interceptors: Vec<Box<dyn Interceptor + Send + Sync>>) -> ClientInterceptor {
        ClientInterceptor::new(move |mut req: Request<()>| -> Result<Request<()>, Status> {
            for int in interceptors.iter_mut() {
                req = int.call(req)?;
            }
            Ok(req)
        })
    }
}
