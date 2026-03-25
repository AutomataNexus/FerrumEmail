//! # ferrum-email-render
//!
//! The rendering engine for Ferrum Email.
//!
//! This crate takes `Component` implementations, calls their `render()` methods to
//! produce `Node` trees, and converts those trees into:
//!
//! - **HTML** — email-safe HTML with all CSS inlined as `style=""` attributes
//! - **Plain text** — extracted from text nodes with sensible formatting
//!
//! ## Rendering Pipeline
//!
//! ```text
//! Component::render() → Node tree
//!        ↓
//!    CSS Inliner       (all styles → inline style="" attrs)
//!        ↓
//!    HTML Emitter      (produces final HTML string)
//!        ↓
//!    Text Extractor    (produces plain text fallback)
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use ferrum_email_render::Renderer;
//! use ferrum_email_core::Component;
//! # struct MyEmail;
//! # impl Component for MyEmail {
//! #     fn render(&self) -> ferrum_email_core::Node { ferrum_email_core::Node::None }
//! # }
//!
//! let renderer = Renderer::default();
//! let email = MyEmail;
//! let html = renderer.render_html(&email).unwrap();
//! let text = renderer.render_text(&email).unwrap();
//! ```

pub mod css_inliner;
pub mod html_emitter;
pub mod renderer;
pub mod text_extractor;

pub use renderer::{RenderConfig, Renderer};

/// Errors that can occur during rendering.
#[derive(Debug)]
pub enum RenderError {
    /// A rendering operation failed.
    RenderFailed(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::RenderFailed(msg) => write!(f, "render error: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}
