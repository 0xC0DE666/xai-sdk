//! Re-exports of commonly used types from dependencies.
//!
//! Curated re-exports so callers can depend on `xai-sdk` without adding `tonic` or
//! `futures` for typical gRPC and streaming use cases. Paths mirror the upstream crates.

/// Core gRPC types re-exported from `tonic`.
pub mod tonic {
    /// - [`Request`] - Wrapper for gRPC request messages
    /// - [`Response`] - Wrapper for gRPC response messages
    /// - [`Status`] - gRPC status codes and error information
    /// - [`Streaming`] - Stream of gRPC response messages
    pub use tonic::{Request, Response, Status, Streaming};

    /// gRPC transport types re-exported from `tonic::transport`.
    pub mod transport {
        pub use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Error};
    }

    /// gRPC service utilities re-exported from `tonic::service`.
    pub mod service {
        pub use tonic::service::Interceptor;

        pub mod interceptor {
            pub use tonic::service::interceptor::InterceptedService;
        }
    }

    /// gRPC metadata types re-exported from `tonic::metadata`.
    pub mod metadata {
        pub use tonic::metadata::MetadataValue;
    }
}

/// Futures types for [`crate::chat::stream::process`] and related streaming helpers.
pub mod futures {
    pub use futures::{SinkExt, StreamExt};

    pub mod stream {
        pub use futures::stream::{self, Stream, iter};
    }

    pub mod channel {
        pub mod mpsc {
            pub use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
        }
    }

    pub mod sink {
        pub use futures::sink::{Sink, drain};
    }
}
