//! The `<head>` component with meta charset, viewport, and optional title.

use ferrum_email_core::*;

/// The `<head>` section of an email.
///
/// Automatically includes meta charset UTF-8, viewport meta for mobile,
/// and content-type meta. Optionally includes a title element.
pub struct Head {
    pub title: Option<String>,
}

impl Head {
    /// Create a new Head component with no title.
    pub fn new() -> Self {
        Head { title: None }
    }

    /// Set the title element.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Head {
    fn default() -> Self {
        Head::new()
    }
}

impl Component for Head {
    fn render(&self) -> Node {
        let mut children: Vec<Node> = vec![
            Node::Element(
                Element::new(Tag::Meta)
                    .attr("http-equiv", "Content-Type")
                    .attr("content", "text/html; charset=UTF-8"),
            ),
            Node::Element(
                Element::new(Tag::Meta)
                    .attr("name", "viewport")
                    .attr("content", "width=device-width, initial-scale=1.0"),
            ),
            Node::Element(
                Element::new(Tag::Meta)
                    .attr("http-equiv", "X-UA-Compatible")
                    .attr("content", "IE=edge"),
            ),
        ];

        if let Some(ref title) = self.title {
            children.push(Node::Element(
                Element::new(Tag::Title).child(Node::text(title)),
            ));
        }

        Node::Element(Element::new(Tag::Head).children(children))
    }
}
