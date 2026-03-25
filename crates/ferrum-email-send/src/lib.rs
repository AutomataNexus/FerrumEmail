//! # ferrum-email-send
//!
//! Email provider abstraction and sending API for Ferrum Email.
//!
//! This crate provides the `Sender` struct — the main API for sending emails —
//! and the `EmailProvider` trait for implementing email backends.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ferrum_email_send::{Sender, providers::ConsoleProvider};
//!
//! # async fn example() -> Result<(), ferrum_email_send::EmailError> {
//! let sender = Sender::new(
//!     ConsoleProvider::new(),
//!     "me@example.com",
//! );
//! // sender.send(&my_email, "recipient@example.com").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Built-in Providers
//!
//! - **ConsoleProvider** — prints to stdout (always available, for development)
//!
//! ## Vault Integration (optional)
//!
//! Enable the `vault` feature to store email credentials securely via NexusVault (AegisVault):
//!
//! ```toml
//! ferrum-email-send = { version = "0.1", features = ["vault"] }
//! ```

pub mod error;
pub mod message;
pub mod provider;
pub mod providers;
pub mod sender;

#[cfg(feature = "vault")]
pub mod vault;

pub use error::EmailError;
pub use message::{Attachment, EmailMessage, EmailTag, Mailbox, SendResult};
pub use provider::EmailProvider;
pub use sender::Sender;
