//! Code components — inline code span and code block.

use ferrum_email_core::*;

/// An inline code span.
///
/// Renders as a `<code>` element with monospace font and subtle background.
pub struct Code {
    pub content: String,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub font_size: Option<Px>,
    pub border_radius: Option<Px>,
    pub padding: Option<Spacing>,
}

impl Code {
    /// Create a new inline Code span.
    pub fn new(content: &str) -> Self {
        Code {
            content: content.to_string(),
            color: None,
            background_color: Some(Color::hex("f4f4f4")),
            font_size: Some(Px(14)),
            border_radius: Some(Px(3)),
            padding: Some(Spacing::xy(Px(2), Px(4))),
        }
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, size: Px) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Code {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.font_family = Some(FontFamily::Monospace);
        style.color = self.color.clone();
        style.background_color = self.background_color.clone();
        style.font_size = self.font_size;
        style.border_radius = self.border_radius;
        style.padding = self.padding;

        Node::Element(
            Element::new(Tag::Code)
                .style(style)
                .child(Node::text(&self.content)),
        )
    }
}

/// A code block with monospace font and background.
///
/// Renders as a `<pre><code>` block suitable for displaying code snippets in emails.
pub struct CodeBlock {
    pub content: String,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub font_size: Option<Px>,
    pub line_height: Option<LineHeight>,
    pub padding: Option<Spacing>,
    pub border_radius: Option<Px>,
}

impl CodeBlock {
    /// Create a new CodeBlock.
    pub fn new(content: &str) -> Self {
        CodeBlock {
            content: content.to_string(),
            color: Some(Color::hex("212121")),
            background_color: Some(Color::hex("f4f4f4")),
            font_size: Some(Px(13)),
            line_height: Some(LineHeight::Multiplier(1.5)),
            padding: Some(Spacing::all(Px(16))),
            border_radius: Some(Px(4)),
        }
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, size: Px) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set the line height.
    pub fn line_height(mut self, height: impl Into<LineHeight>) -> Self {
        self.line_height = Some(height.into());
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

impl Component for CodeBlock {
    fn render(&self) -> Node {
        let mut pre_style = Style::new();
        pre_style.background_color = self.background_color.clone();
        pre_style.padding = self.padding;
        pre_style.border_radius = self.border_radius;
        pre_style.margin = Some(Spacing::xy(Px(16), Px(0)));

        let mut code_style = Style::new();
        code_style.font_family = Some(FontFamily::Monospace);
        code_style.color = self.color.clone();
        code_style.font_size = self.font_size;
        code_style.line_height = self.line_height;

        Node::Element(
            Element::new(Tag::Pre).style(pre_style).child(Node::Element(
                Element::new(Tag::Code)
                    .style(code_style)
                    .child(Node::text(&self.content)),
            )),
        )
    }
}
