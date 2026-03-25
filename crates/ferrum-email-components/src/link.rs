//! Link (anchor) component.

use ferrum_email_core::*;

/// An anchor/link element.
///
/// Renders as an `<a>` tag with configurable color, text decoration, and font properties.
pub struct Link {
    pub href: String,
    pub text: String,
    pub color: Option<Color>,
    pub font_size: Option<Px>,
    pub font_weight: Option<FontWeight>,
    pub text_decoration: Option<TextDecoration>,
    pub target: String,
}

impl Link {
    /// Create a new Link with href and display text.
    pub fn new(href: &str, text: &str) -> Self {
        Link {
            href: href.to_string(),
            text: text.to_string(),
            color: Some(Color::hex("067df7")),
            font_size: None,
            font_weight: None,
            text_decoration: Some(TextDecoration::Underline),
            target: "_blank".to_string(),
        }
    }

    /// Set the link color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, size: Px) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set the font weight.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Set the text decoration.
    pub fn text_decoration(mut self, decoration: TextDecoration) -> Self {
        self.text_decoration = Some(decoration);
        self
    }

    /// Set the target attribute.
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Link {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.color = self.color.clone();
        style.font_size = self.font_size;
        style.font_weight = self.font_weight;
        style.text_decoration = self.text_decoration;

        Node::Element(
            Element::new(Tag::A)
                .attr("href", &self.href)
                .attr("target", &self.target)
                .style(style)
                .child(Node::text(&self.text)),
        )
    }
}
