//! The `Component` trait — the core abstraction of Ferrum Email.
//!
//! Every email template and reusable email element implements this trait.

use crate::node::Node;

/// The core trait for all Ferrum Email components.
///
/// Components are composable building blocks for email templates. Each component
/// defines how it renders to a `Node` tree, which the renderer then converts
/// to email-safe HTML.
///
/// # Examples
///
/// ```
/// use ferrum_email_core::{Component, Node, node::Element, node::Tag};
///
/// struct HelloEmail {
///     name: String,
/// }
///
/// impl Component for HelloEmail {
///     fn render(&self) -> Node {
///         Node::text(format!("Hello, {}!", self.name))
///     }
///
///     fn subject(&self) -> Option<&str> {
///         Some("Hello!")
///     }
/// }
/// ```
pub trait Component: Send + Sync {
    /// Render this component into a `Node` tree.
    fn render(&self) -> Node;

    /// Optional: provide a custom plain text version.
    ///
    /// If `None`, the renderer will auto-extract plain text from the node tree.
    fn plain_text(&self) -> Option<String> {
        None
    }

    /// The email subject line. Only meaningful on top-level email components.
    fn subject(&self) -> Option<&str> {
        None
    }
}
