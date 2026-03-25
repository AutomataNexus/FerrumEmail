//! CTA Button component using table-based layout for Outlook compatibility.

use ferrum_email_core::*;

/// A call-to-action button for emails.
///
/// Renders using a table-based approach for maximum compatibility across
/// email clients, including Microsoft Outlook which uses the Word rendering engine.
pub struct Button {
    pub href: String,
    pub label: String,
    pub background_color: Color,
    pub text_color: Color,
    pub border_radius: Px,
    pub padding: Spacing,
    pub font_size: Px,
    pub font_weight: FontWeight,
    pub font_family: Option<FontFamily>,
    pub text_align: Option<TextAlign>,
}

impl Button {
    /// Create a new Button with an href and label.
    pub fn new(href: &str, label: &str) -> Self {
        Button {
            href: href.to_string(),
            label: label.to_string(),
            background_color: Color::hex("000000"),
            text_color: Color::hex("ffffff"),
            border_radius: Px(4),
            padding: Spacing::xy(Px(12), Px(20)),
            font_size: Px(14),
            font_weight: FontWeight::SemiBold,
            font_family: Some(FontFamily::SansSerif),
            text_align: None,
        }
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the border radius.
    pub fn border_radius(mut self, radius: Px) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set the padding.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, size: Px) -> Self {
        self.font_size = size;
        self
    }

    /// Set the font weight.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set the font family.
    pub fn font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    /// Set the container text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Button {
    fn default() -> Self {
        Button::new("", "")
    }
}

impl Component for Button {
    fn render(&self) -> Node {
        // Table-based button layout for Outlook compatibility:
        // <table><tbody><tr><td style="background;border-radius;padding">
        //   <a href="..." style="color;font;text-decoration:none;display:inline-block">Label</a>
        // </td></tr></tbody></table>

        let mut td_style = Style::new();
        td_style.background_color = Some(self.background_color.clone());
        td_style.border_radius = Some(self.border_radius);
        td_style.padding = Some(self.padding);
        td_style.text_align = Some(TextAlign::Center);

        let mut a_style = Style::new();
        a_style.color = Some(self.text_color.clone());
        a_style.font_size = Some(self.font_size);
        a_style.font_weight = Some(self.font_weight);
        a_style.font_family = self.font_family.clone();
        a_style.text_decoration = Some(TextDecoration::None);
        a_style.display = Some(Display::InlineBlock);

        let link = Element::new(Tag::A)
            .attr("href", &self.href)
            .attr("target", "_blank")
            .style(a_style)
            .child(Node::text(&self.label));

        let mut table_style = Style::new();
        if let Some(align) = self.text_align {
            table_style.text_align = Some(align);
        }

        Node::Element(
            Element::new(Tag::Table)
                .attr("role", "presentation")
                .attr("cellpadding", "0")
                .attr("cellspacing", "0")
                .attr("border", "0")
                .style(table_style)
                .child(Node::Element(
                    Element::new(Tag::Tbody).child(Node::Element(
                        Element::new(Tag::Tr).child(Node::Element(
                            Element::new(Tag::Td)
                                .style(td_style)
                                .child(Node::Element(link)),
                        )),
                    )),
                )),
        )
    }
}
