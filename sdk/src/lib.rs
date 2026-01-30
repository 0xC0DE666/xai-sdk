//! xAI SDK for Rust
//!
//! A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services
//! including Grok language models, embeddings, image generation, and more.

/// Default xAI API URL
pub const XAI_API_URL: &str = "https://api.x.ai:443";

pub mod api;
pub mod auth;
pub mod billing;
pub mod chat;
pub mod common;
pub mod documents;
pub mod embed;
pub mod export;
pub mod image;
pub mod models;
pub mod sample;
pub mod tokenize;
pub mod utils;
pub mod video;
pub use export::*;

// Generated proto files (from build.rs)
#[path = "prod.clickhouse_analytics.rs"]
mod clickhouse_analytics;
mod prod_charger;
mod prod_mc_billing;
mod xai_api;

#[allow(missing_docs)]
pub(crate) mod prod {
    pub(crate) use crate::clickhouse_analytics;
}
