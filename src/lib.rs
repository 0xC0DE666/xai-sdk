//! xAI SDK for Rust
//!
//! A Rust SDK for xAI's API, providing type-safe gRPC clients for all xAI services
//! including Grok language models, embeddings, image generation, and more.

pub mod auth;
pub mod chat;
pub mod common;
pub mod documents;
pub mod embed;
pub mod image;
pub mod models;
pub mod sample;
pub mod tokenize;
pub mod utils;
pub mod xai_api;

/// Default xAI API URL
pub const XAI_API_URL: &str = "https://api.x.ai:443";
