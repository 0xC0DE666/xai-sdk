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
pub use export::*;

// Generated proto files (from build.rs)
#[allow(missing_docs)]
mod prod_charger;
#[allow(missing_docs)]
mod prod_mc_billing;
#[allow(missing_docs)]
mod xai_api;
#[allow(missing_docs)]
mod prod {
    // Generated proto module
    #[allow(missing_docs)]
    #[path = "prod.clickhouse_analytics.rs"]
    pub mod clickhouse_analytics;
}
