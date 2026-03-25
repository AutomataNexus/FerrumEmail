//! Heading component (H1-H6) with size presets.

use ferrum_email_core::*;

/// A heading element (H1-H6) with typography controls.
///
/// Each heading level has default font sizes that can be overridden.
pub struct Heading {
    pub level: HeadingLevel,
    pub content: String,
    pub color: Option<Color>,
    pub font_size: Option<Px>,
    pub font_weight: Option<FontWeight>,
    pub font_family: Option<FontFamily>,
    pub line_height: Option<LineHeight>,
    pub text_align: Option<TextAlign>,
    pub margin: Option<Spacing>,
}

impl Heading {
    /// Create a new Heading with the given level and content.
    pub fn new(level: HeadingLevel, content: &str) -> Self {
        Heading {
            level,
            content: content.to_string(),
            color: None,
            font_size: None, // Uses level default
            font_weight: Some(FontWeight::Bold),
            font_family: None,
            line_height: Some(LineHeight::Multiplier(1.3)),
            text_align: None,
            margin: Some(Spacing::xy(Px(16), Px(0))),
        }
    }

    /// Create an H1 heading.
    pub fn h1(content: &str) -> Self {
        Heading::new(HeadingLevel::H1, content)
    }

    /// Create an H2 heading.
    pub fn h2(content: &str) -> Self {
        Heading::new(HeadingLevel::H2, content)
    }

    /// Create an H3 heading.
    pub fn h3(content: &str) -> Self {
        Heading::new(HeadingLevel::H3, content)
    }

    /// Create an H4 heading.
    pub fn h4(content: &str) -> Self {
        Heading::new(HeadingLevel::H4, content)
    }

    /// Create an H5 heading.
    pub fn h5(content: &str) -> Self {
        Heading::new(HeadingLevel::H5, content)
    }

    /// Create an H6 heading.
    pub fn h6(content: &str) -> Self {
        Heading::new(HeadingLevel::H6, content)
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the font size (overrides the level default).
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

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Heading {
    fn render(&self) -> Node {
        let tag = match self.level {
            HeadingLevel::H1 => Tag::H1,
            HeadingLevel::H2 => Tag::H2,
            HeadingLevel::H3 => Tag::H3,
            HeadingLevel::H4 => Tag::H4,
            HeadingLevel::H5 => Tag::H5,
            HeadingLevel::H6 => Tag::H6,
        };

        let mut style = Style::new();
        style.color = self.color.clone();
        style.font_size = Some(
            self.font_size
                .unwrap_or_else(|| self.level.default_font_size()),
        );
        style.font_weight = self.font_weight;
        style.font_family = self.font_family.clone();
        style.line_height = self.line_height;
        style.text_align = self.text_align;
        style.margin = self.margin;

        Node::Element(
            Element::new(tag)
                .style(style)
                .child(Node::text(&self.content)),
        )
    }
}
