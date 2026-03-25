//! Horizontal rule divider component.

use ferrum_email_core::*;

/// A horizontal rule divider.
///
/// Renders as an `<hr>` element with configurable color, width, and margin.
pub struct Hr {
    pub color: Option<Color>,
    pub width: Option<SizeValue>,
    pub margin: Option<Spacing>,
    pub border_style: BorderStyle,
}

impl Hr {
    /// Create a new horizontal rule.
    pub fn new() -> Self {
        Hr {
            color: Some(Color::hex("eaeaea")),
            width: Some(SizeValue::Percent(Percent(100.0))),
            margin: Some(Spacing::xy(Px(16), Px(0))),
            border_style: BorderStyle::Solid,
        }
    }

    /// Set the rule color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the rule width.
    pub fn width(mut self, width: SizeValue) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the margin.
    pub fn margin(mut self, margin: Spacing) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Hr {
    fn default() -> Self {
        Hr::new()
    }
}

impl Component for Hr {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.width = self.width;
        style.margin = self.margin;

        if let Some(ref color) = self.color {
            style.border = Some(Border::new(Px(0), BorderStyle::None, Color::transparent()));
            style.border_top = Some(Border::new(Px(1), self.border_style, color.clone()));
        }

        Node::Element(Element::new(Tag::Hr).style(style))
    }
}
