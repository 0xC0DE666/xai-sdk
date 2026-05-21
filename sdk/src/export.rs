//! Re-exports of commonly used types from dependencies.
//!
//! Curated re-exports so callers can depend on `xai-sdk` without adding `tonic` or
//! `futures` for typical gRPC, actor-mailbox, RPC-reply, and streaming use cases.
//! Paths mirror the upstream crates.

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

/// Futures types for service mailboxes, RPC replies, and chat streaming.
///
/// Typical layout:
/// - [`channel::mpsc`] — bounded actor mailboxes (`channel(100)`), [`SinkExt::send`], [`StreamExt::next`]
/// - [`channel::oneshot`] — single RPC reply (`reply_to` in request structs)
/// - [`stream::Stream`] — chunk streams from transports; [`stream::iter`] for test mocks
/// - [`SinkExt`] / [`StreamExt`] — async `.send().await`, `.next().await`, receiver close
pub mod futures {
    pub use futures::{FutureExt, SinkExt, StreamExt};

    pub mod stream {
        pub use futures::stream::{self, Stream, iter};
    }

    pub mod channel {
        /// Multi-producer single-consumer mailboxes (bounded and unbounded).
        pub mod mpsc {
            pub use futures::channel::mpsc::{
                Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError,
                UnboundedReceiver, UnboundedSender, channel, unbounded,
            };
        }

        /// Single-use reply channels for request/response RPC.
        pub mod oneshot {
            pub use futures::channel::oneshot::{
                Canceled, Cancellation, Receiver, Sender, channel,
            };
        }
    }

    pub mod sink {
        pub use futures::sink::{Sink, drain};
    }
}
