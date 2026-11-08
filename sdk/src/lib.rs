//! xAI SDK for Rust
//!
//! A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services
//! including Grok language models, embeddings, image generation, and more.

/// Default xAI inference API URL (chat, models, embeddings, …).
pub const XAI_API_URL: &str = "https://api.x.ai:443";

/// Default xAI management API URL (billing and team management).
///
/// Requires a management key from the xAI console — inference API keys are rejected.
pub const XAI_MANAGEMENT_API_URL: &str = "https://management-api.x.ai:443";

/// Environment variable for the inference API key (`api.x.ai`).
pub const XAI_API_KEY: &str = "XAI_API_KEY";

/// Environment variable for the management API key (`management-api.x.ai`).
pub const XAI_MANAGEMENT_API_KEY: &str = "XAI_MANAGEMENT_API_KEY";

/// Environment variable for the team UUID (required on management billing RPCs).
pub const XAI_TEAM_ID: &str = "XAI_TEAM_ID";

pub mod api;
pub mod auth;
pub mod batch;
pub mod billing;
pub mod chat;
pub mod shared;
pub mod documents;
pub mod embed;
pub mod export;
pub mod files;
pub mod image;
pub mod models;
#[deprecated(
    since = "0.13.0",
    note = "Raw SampleText is rejected for current Grok 4 models; use `chat` instead"
)]
pub mod sample;
pub mod tokenize;
pub mod utils;
pub mod video;

pub(crate) mod google {
    pub mod rpc {
        include!("google.rpc.rs");
    }
}

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
