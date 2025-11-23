//! Re-exports of commonly used types from dependencies.
//!
//! This module provides convenient re-exports of frequently used types from `tonic`
//! so users don't need to add `tonic` as a dependency themselves.

pub use tonic::{Request, Response, Status, Streaming};
