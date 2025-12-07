//! xAI SDK for Rust
//!
//! A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services
//! including Grok language models, embeddings, image generation, and more.

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
pub mod xai_api;

// Generated proto files (from build.rs)
#[allow(missing_docs)]
pub mod prod_charger;
#[allow(missing_docs)]
pub mod prod_mc_billing;
#[allow(missing_docs)]
pub mod prod;

// Generated proto files - organized for cleaner API
#[doc(hidden)]
pub mod generated {
    // Main API protos
    pub mod api {
        pub use crate::xai_api::*;
    }

    // Billing/management API protos
    pub mod billing {
        pub use crate::prod_mc_billing::*;
    }

    // Shared billing types
    pub mod charger {
        pub use crate::prod_charger::*;
    }

    // Analytics
    pub mod analytics {
        pub use crate::prod::clickhouse_analytics::*;
    }
}

pub use export::*;

/// Default xAI API URL
pub const XAI_API_URL: &str = "https://api.x.ai:443";
