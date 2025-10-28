pub mod auth;
pub mod chat;
pub mod documents;
pub mod embed;
pub mod image;
pub mod models;
pub mod sample;
pub mod tokenize;
mod xai_api;

pub use xai_api::*;

/// Default xAI API URL
pub const XAI_API_URL: &str = "https://api.x.ai:443";
