//! Spacer component for vertical whitespace.

use ferrum_email_core::*;

/// A vertical spacer that creates whitespace between elements.
///
/// Renders as a table with an empty `<td>` of the specified height.
/// Using a table ensures consistent spacing across email clients.
pub struct Spacer {
    pub height: Px,
}

impl Spacer {
    /// Create a new Spacer with the given height.
    pub fn new(height: Px) -> Self {
        Spacer { height }
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Spacer {
    fn render(&self) -> Node {
        let mut td_style = Style::new();
        td_style.height = Some(SizeValue::Px(self.height));
        td_style.line_height = Some(LineHeight::Px(self.height));
        td_style.font_size = Some(Px(1));

        Node::Element(
            Element::new(Tag::Table)
                .attr("role", "presentation")
                .attr("cellpadding", "0")
                .attr("cellspacing", "0")
                .attr("border", "0")
                .attr("width", "100%")
                .child(Node::Element(
                    Element::new(Tag::Tbody).child(Node::Element(
                        Element::new(Tag::Tr).child(Node::Element(
                            Element::new(Tag::Td)
                                .attr("height", format!("{}", self.height.0))
                                .style(td_style)
                                .child(Node::text("\u{00A0}")),
                        )),
                    )),
                )),
        )
    }
}
