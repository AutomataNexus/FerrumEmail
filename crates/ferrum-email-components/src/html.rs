//! The root `<html>` component for email templates.

use ferrum_email_core::*;

/// The root HTML element for an email.
///
/// Wraps the entire email content in `<html>` with proper lang and dir attributes.
/// The DOCTYPE is handled by the renderer, not this component.
pub struct Html {
    pub lang: String,
    pub dir: String,
    pub children: Vec<Node>,
}

impl Html {
    /// Create a new Html component with default lang="en" and dir="ltr".
    pub fn new() -> Self {
        Html {
            lang: "en".to_string(),
            dir: "ltr".to_string(),
            children: Vec::new(),
        }
    }

    /// Set the language attribute.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Set the text direction attribute.
    pub fn dir(mut self, dir: impl Into<String>) -> Self {
        self.dir = dir.into();
        self
    }

    /// Add a child component.
    pub fn child(mut self, child: impl Component) -> Self {
        self.children.push(child.render());
        self
    }

    /// Add a raw Node as a child.
    pub fn child_node(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Html {
    fn default() -> Self {
        Html::new()
    }
}

impl Component for Html {
    fn render(&self) -> Node {
        let element = Element::new(Tag::Html)
            .attr("lang", &self.lang)
            .attr("dir", &self.dir)
            .attr("xmlns", "http://www.w3.org/1999/xhtml")
            .children(self.children.clone());
        Node::Element(element)
    }
}
