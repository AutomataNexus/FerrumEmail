//! Text/paragraph component with typography props.

use ferrum_email_core::*;

/// A paragraph of text with typography controls.
///
/// Renders as a `<p>` element with inline styles for font size, color,
/// line height, font weight, and more.
pub struct Text {
    pub content: String,
    pub color: Option<Color>,
    pub font_size: Option<Px>,
    pub font_weight: Option<FontWeight>,
    pub font_family: Option<FontFamily>,
    pub line_height: Option<LineHeight>,
    pub text_align: Option<TextAlign>,
    pub margin: Option<Spacing>,
    pub padding: Option<Spacing>,
}

impl Text {
    /// Create a new Text component with the given content.
    pub fn new(content: &str) -> Self {
        Text {
            content: content.to_string(),
            color: None,
            font_size: Some(Px(14)),
            font_weight: None,
            font_family: None,
            line_height: Some(LineHeight::Multiplier(1.5)),
            text_align: None,
            margin: Some(Spacing::xy(Px(0), Px(0))),
            padding: None,
        }
    }

    /// Set the text color.
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

    /// Set the font family.
    pub fn font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    /// Set the line height.
    pub fn line_height(mut self, height: impl Into<LineHeight>) -> Self {
        self.line_height = Some(height.into());
        self
    }

    /// Set the text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Set the margin.
    pub fn margin(mut self, margin: Spacing) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set the padding.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Text {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.color = self.color.clone();
        style.font_size = self.font_size;
        style.font_weight = self.font_weight;
        style.font_family = self.font_family.clone();
        style.line_height = self.line_height;
        style.text_align = self.text_align;
        style.margin = self.margin;
        style.padding = self.padding;

        Node::Element(
            Element::new(Tag::P)
                .style(style)
                .child(Node::text(&self.content)),
        )
    }
}
