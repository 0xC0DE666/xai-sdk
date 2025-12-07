//! API organization module.
//!
//! This module provides a clean, organized public API surface that groups related
//! functionality together. It re-exports generated proto types in a logical structure
//! that matches the xAI API organization.
//!
//! # Structure
//!
//! - **Standard API**: Main xAI API types (chat, embeddings, images, etc.)
//! - **Management API**: Team and account management APIs
//!   - **Billing**: Billing service, payment methods, invoices, and spending limits
//!   - **Analytics**: Analytics types for querying billing and usage data

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
