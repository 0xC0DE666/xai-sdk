//! API organization module.
//!
//! Provides a clean, organized public API surface that groups related functionality.
//! Re-exports generated Protocol Buffer types in a logical structure matching xAI API organization.
//!
//! # Structure
//!
//! ## Standard API
//! Core xAI API types for AI services:
//! - **Chat**: [`GetCompletionsRequest`], [`GetChatCompletionResponse`], [`Message`], etc.
//! - **Embeddings**: [`EmbedRequest`], [`EmbedResponse`], embedding types
//! - **Images**: [`GenerateImageRequest`], [`GenerateImageResponse`], image types
//! - **Models**: [`ListLanguageModelsRequest`], model information types
//! - **Sample**: [`SampleTextRequest`], text generation types
//! - **Tokenize**: [`TokenizeRequest`], tokenization types
//! - **Documents**: [`SearchDocumentsRequest`], document search types
//! - **Auth**: [`GetApiKeyInfoRequest`], authentication types
//!
//! ## Management API
//! Team and account management APIs in [`management`] module:
//! - **Billing**: Billing service types in [`management::billing`]
//!   - Payment methods, invoices, spending limits, prepaid credits
//!   - Usage analytics and billing information

// Standard API
pub use crate::xai_api::*;

// Management API
pub mod management {
    pub mod billing {
        pub use crate::prod_charger::*;
        pub use crate::prod_mc_billing::*;

        pub mod analytics {
            pub use crate::prod::clickhouse_analytics::*;
        }
    }
}
