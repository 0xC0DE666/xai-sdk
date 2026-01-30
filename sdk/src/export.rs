//! Re-exports of commonly used types from dependencies.
//!
//! Provides convenient re-exports of frequently used types from `tonic` and other
//! dependencies, so users don't need to add these crates as direct dependencies.
//!
//! Module structure mirrors `tonic`'s organization for familiarity.

/// Core gRPC types re-exported from `tonic`.
///
/// - [`Request`] - Wrapper for gRPC request messages
/// - [`Response`] - Wrapper for gRPC response messages
/// - [`Status`] - gRPC status codes and error information
/// - [`Streaming`] - Stream of gRPC response messages
pub use tonic::{Request, Response, Status, Streaming};

/// gRPC transport types re-exported from `tonic::transport`.
///
/// - [`Channel`] - gRPC connection channel
/// - [`ClientTlsConfig`] - TLS configuration for secure connections
/// - [`Endpoint`] - gRPC endpoint configuration
/// - [`Error`] - Transport-level errors
pub mod transport {
    pub use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Error};
}

/// gRPC service utilities re-exported from `tonic::service`.
///
/// - [`Interceptor`] - Request/response interception trait
/// - [`interceptor::InterceptedService`] - Service with interceptors applied
pub mod service {
    pub use tonic::service::Interceptor;

    pub mod interceptor {
        pub use tonic::service::interceptor::InterceptedService;
    }
}

/// gRPC metadata types re-exported from `tonic::metadata`.
///
/// - [`MetadataValue`] - HTTP header/metadata values
pub mod metadata {
    pub use tonic::metadata::MetadataValue;
}
