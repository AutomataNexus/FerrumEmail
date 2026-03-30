//! # Ferrum Email
//!
//! **The complete email framework for Rust.** One crate, everything included.
//!
//! Ferrum Email is the Rust equivalent of React Email + Resend: type-safe,
//! composable email templates with cross-client-compatible HTML rendering
//! and a unified async sending API.
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! ferrum-email = "0.1"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ```rust,no_run
//! use ferrum_email::prelude::*;
//!
//! struct WelcomeEmail { name: String }
//!
//! impl Component for WelcomeEmail {
//!     fn subject(&self) -> Option<&str> { Some("Welcome!") }
//!
//!     fn render(&self) -> Node {
//!         Html::new()
//!             .child(Body::new().child(
//!                 Container::new().child(
//!                     Section::new().padding(Spacing::all(Px(32)))
//!                         .child_node(
//!                             Heading::h1(&format!("Hello, {}!", self.name))
//!                                 .color(Color::hex("2D2A26"))
//!                                 .into_node())
//!                         .child_node(
//!                             Button::new("https://example.com", "Get Started")
//!                                 .background(Color::hex("C0582B"))
//!                                 .text_color(Color::white())
//!                                 .into_node())
//!                 )))
//!             .into_node()
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Send via the Ferrum Mail platform
//! let provider = SmtpProvider::builder()
//!     .host("ferrum-mail.com")
//!     .port(587)
//!     .credentials("fm_your_api_key", "")
//!     .build()?;
//!
//! let sender = Sender::new(provider, "you@yourapp.com");
//! sender.send(&WelcomeEmail { name: "World".into() }, "user@example.com").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ferrum-email (this crate — umbrella SDK)
//!  ├── ferrum-email-core        — Component trait, Node tree, Style system
//!  ├── ferrum-email-components  — Html, Body, Button, Text, etc.
//!  ├── ferrum-email-render      — HTML renderer, CSS inliner, plain text
//!  └── ferrum-email-send        — Sender, SMTP provider, NexusShield
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `smtp` | Yes | Native SMTP provider with STARTTLS |
//! | `shield` | Yes | NexusShield email security validation |
//! | `vault` | No | NexusVault encrypted credential storage |
//!
//! ## Sending via Ferrum Mail Platform
//!
//! Sign up at [ferrum-mail.com](https://ferrum-mail.com) for managed email delivery:
//!
//! ```rust,no_run
//! # use ferrum_email::prelude::*;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = SmtpProvider::builder()
//!     .host("ferrum-mail.com")
//!     .port(587)
//!     .credentials("fm_your_api_key", "")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! Or use your own SMTP server:
//!
//! ```rust,no_run
//! # use ferrum_email::prelude::*;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = SmtpProvider::builder()
//!     .host("smtp.office365.com")
//!     .port(587)
//!     .credentials("user@example.com", "password")
//!     .build()?;
//! # Ok(())
//! # }
//! ```

// Re-export all sub-crates
pub use ferrum_email_core as core;
pub use ferrum_email_components as components;
pub use ferrum_email_render as render;
pub use ferrum_email_send as send;

/// The prelude — import everything you need with one `use` statement.
///
/// ```rust
/// use ferrum_email::prelude::*;
/// ```
pub mod prelude {
    // Core types
    pub use ferrum_email_core::{
        Component, Node, Element, Tag, Attr, Style, Border, BorderStyle,
        Color, Spacing, Px, Percent, SizeValue,
        FontFamily, FontWeight, TextAlign, VerticalAlign,
        LineHeight, Display, TextDecoration, HeadingLevel,
    };

    // All components
    pub use ferrum_email_components::{
        Html, Head, Body, Preview, Container, Section, Row, Column,
        Text, Heading, Button, Link, Image, Hr, Code, CodeBlock, Spacer,
    };

    // Renderer
    pub use ferrum_email_render::{Renderer, RenderConfig, RenderError};

    // Sender and providers
    pub use ferrum_email_send::{
        Sender, EmailProvider, EmailMessage, EmailError,
        Mailbox, Attachment, EmailTag, SendResult,
    };

    // SMTP provider
    #[cfg(feature = "smtp")]
    pub use ferrum_email_send::providers::SmtpProvider;

    // Direct MX delivery (NexusRelay)
    #[cfg(feature = "smtp")]
    pub use ferrum_email_send::providers::DirectMxProvider;

    // Console provider (always available)
    pub use ferrum_email_send::providers::ConsoleProvider;

    // Vault credential store
    #[cfg(feature = "vault")]
    pub use ferrum_email_send::VaultCredentialStore;
}
