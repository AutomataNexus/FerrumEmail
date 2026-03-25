//! Full-width section component.

use ferrum_email_core::*;

/// A full-width section within an email layout.
///
/// Renders as a table row for email client compatibility. Supports
/// background color and padding.
pub struct Section {
    pub background_color: Option<Color>,
    pub padding: Option<Spacing>,
    pub text_align: Option<TextAlign>,
    pub children: Vec<Node>,
}

impl Section {
    /// Create a new Section.
    pub fn new() -> Self {
        Section {
            background_color: None,
            padding: None,
            text_align: None,
            children: Vec::new(),
        }
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set padding.
    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
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

impl Default for Section {
    fn default() -> Self {
        Section::new()
    }
}

impl Component for Section {
    fn render(&self) -> Node {
        let mut td_style = Style::new();
        td_style.background_color = self.background_color.clone();
        td_style.padding = self.padding;
        td_style.text_align = self.text_align;

        Node::Element(
            Element::new(Tag::Table)
                .attr("role", "presentation")
                .attr("cellpadding", "0")
                .attr("cellspacing", "0")
                .attr("border", "0")
                .attr("width", "100%")
                .style({
                    let mut s = Style::new();
                    s.width = Some(SizeValue::Percent(Percent(100.0)));
                    s
                })
                .child(Node::Element(
                    Element::new(Tag::Tbody).child(Node::Element(
                        Element::new(Tag::Tr).child(Node::Element(
                            Element::new(Tag::Td)
                                .style(td_style)
                                .children(self.children.clone()),
                        )),
                    )),
                )),
        )
    }
}
