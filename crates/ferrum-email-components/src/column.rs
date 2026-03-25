//! Table column component for multi-column email layouts.

use ferrum_email_core::*;

/// A table column within a `Row`.
///
/// Renders as a `<td>` element with configurable width, padding, and alignment.
pub struct Column {
    pub width: Option<SizeValue>,
    pub padding: Option<Spacing>,
    pub vertical_align: Option<VerticalAlign>,
    pub text_align: Option<TextAlign>,
    pub background_color: Option<Color>,
    pub children: Vec<Node>,
}

impl Column {
    /// Create a new Column.
    pub fn new() -> Self {
        Column {
            width: None,
            padding: None,
            vertical_align: Some(VerticalAlign::Top),
            text_align: None,
            background_color: None,
            children: Vec::new(),
        }
    }

    /// Set the column width.
    pub fn width(mut self, width: SizeValue) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the column width as a percentage.
    pub fn width_percent(mut self, pct: f32) -> Self {
        self.width = Some(SizeValue::Percent(Percent(pct)));
        self
    }

    /// Set the column width in pixels.
    pub fn width_px(mut self, px: u32) -> Self {
        self.width = Some(SizeValue::Px(Px(px)));
        self
    }

    /// Set padding.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set vertical alignment.
    pub fn vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = Some(align);
        self
    }

    /// Set text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Set background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
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

    /// Add multiple children.
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Default for Column {
    fn default() -> Self {
        Column::new()
    }
}

impl Component for Column {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.width = self.width;
        style.padding = self.padding;
        style.vertical_align = self.vertical_align;
        style.text_align = self.text_align;
        style.background_color = self.background_color.clone();

        let mut element = Element::new(Tag::Td).style(style);

        // Set width attribute as well for Outlook compatibility
        if let Some(ref w) = self.width {
            match w {
                SizeValue::Px(px) => {
                    element = element.attr("width", format!("{}", px.0));
                }
                SizeValue::Percent(pct) => {
                    element = element.attr("width", format!("{}%", pct.0));
                }
                SizeValue::Auto => {}
            }
        }

        if let Some(ref va) = self.vertical_align {
            element = element.attr("valign", va.to_string());
        }

        Node::Element(element.children(self.children.clone()))
    }
}
