//! Image component with required alt, width, and height.

use ferrum_email_core::*;

/// An image element for emails.
///
/// Width and height are required for email client compatibility — many clients
/// will not properly size images without explicit dimensions.
pub struct Image {
    pub src: String,
    pub alt: String,
    pub width: Px,
    pub height: Option<Px>,
    pub border: Option<Border>,
    pub border_radius: Option<Px>,
    pub display: Option<Display>,
}

impl Image {
    /// Create a new Image with src, alt text, and width.
    pub fn new(src: &str, alt: &str, width: Px) -> Self {
        Image {
            src: src.to_string(),
            alt: alt.to_string(),
            width,
            height: None,
            border: None,
            border_radius: None,
            display: Some(Display::Block),
        }
    }

    /// Set the height.
    pub fn height(mut self, height: Px) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the border.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Set the border radius.
    pub fn border_radius(mut self, radius: Px) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set the display mode.
    pub fn display(mut self, display: Display) -> Self {
        self.display = Some(display);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Image {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.width = Some(SizeValue::Px(self.width));
        if let Some(h) = self.height {
            style.height = Some(SizeValue::Px(h));
        }
        style.border = self.border.clone();
        style.border_radius = self.border_radius;
        style.display = self.display;

        let mut element = Element::new(Tag::Img)
            .attr("src", &self.src)
            .attr("alt", &self.alt)
            .attr("width", format!("{}", self.width.0))
            .style(style);

        if let Some(h) = self.height {
            element = element.attr("height", format!("{}", h.0));
        }

        Node::Element(element)
    }
}
