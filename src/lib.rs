//! xAI SDK for Rust
//!
//! A comprehensive Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services
//! including Grok language models, embeddings, image generation, and more.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use xai_sdk::chat;
//! use tonic::Request;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = chat::client::new("your-api-key").await?;
//! // Use the client...
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod chat;
pub mod common;
pub mod documents;
pub mod embed;
pub mod image;
pub mod models;
pub mod sample;
pub mod tokenize;
pub mod xai_api;

/// Default xAI API URL
pub const XAI_API_URL: &str = "https://api.x.ai:443";
