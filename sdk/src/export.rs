//! Re-exports of commonly used types from dependencies.
//!
//! This module provides convenient re-exports of frequently used types from `tonic`
//! so users don't need to add `tonic` as a dependency themselves.
//!
//! The module structure mirrors `tonic`'s organization for familiarity.

// Core types
pub use tonic::{Request, Response, Status, Streaming};

// Transport module
pub mod transport {
    pub use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Error};
}

// Service module
pub mod service {
    pub use tonic::service::Interceptor;

    pub mod interceptor {
        pub use tonic::service::interceptor::InterceptedService;
    }
}

// Metadata module
pub mod metadata {
    pub use tonic::metadata::MetadataValue;
}
