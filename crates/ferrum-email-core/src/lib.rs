//! # ferrum-email-core
//!
//! The foundation of the Ferrum Email framework. This crate defines the core abstractions:
//!
//! - **`Component`** trait — the building block for all email templates
//! - **`Node`** tree — the intermediate representation between components and rendered HTML
//! - **`Style`** system — type-safe CSS properties for email-safe styling
//! - **Primitive types** — `Px`, `Color`, `Spacing`, `FontWeight`, etc.
//!
//! All other Ferrum Email crates depend on this one. If you're building custom
//! components or extending the framework, this is your entry point.
//!
//! # Architecture
//!
//! ```text
//! Component::render() → Node tree → Renderer → HTML + Plain Text
//! ```
//!
//! Components produce `Node` trees. The `Node` tree is a virtual DOM-like
//! intermediate representation that the renderer walks to produce email-safe HTML.

pub mod color;
pub mod component;
pub mod node;
pub mod spacing;
pub mod style;
pub mod types;

// Re-export core types at the crate root for ergonomic use.
pub use color::Color;
pub use component::Component;
pub use node::{Attr, Element, Node, Tag};
pub use spacing::Spacing;
pub use style::{Border, Style};
pub use types::*;
