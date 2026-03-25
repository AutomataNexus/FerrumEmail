//! The `<body>` component with background color and base styling.

use ferrum_email_core::*;

/// The `<body>` element of an email.
///
/// Supports background color and base font family. All email content
/// goes inside this component.
pub struct Body {
    pub background_color: Option<Color>,
    pub font_family: Option<FontFamily>,
    pub margin: Option<Spacing>,
    pub padding: Option<Spacing>,
    pub children: Vec<Node>,
}

impl Body {
    /// Create a new Body component.
    pub fn new() -> Self {
        Body {
            background_color: None,
            font_family: Some(FontFamily::SansSerif),
            margin: Some(Spacing::zero()),
            padding: Some(Spacing::zero()),
            children: Vec::new(),
        }
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set the font family.
    pub fn font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    /// Set margin.
    pub fn margin(mut self, margin: Spacing) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set padding.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = Some(padding);
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

    /// Add multiple children from an array of Nodes.
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::new()
    }
}

impl Component for Body {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.background_color = self.background_color.clone();
        style.font_family = self.font_family.clone();
        style.margin = self.margin;
        style.padding = self.padding;

        Node::Element(
            Element::new(Tag::Body)
                .style(style)
                .children(self.children.clone()),
        )
    }
}
