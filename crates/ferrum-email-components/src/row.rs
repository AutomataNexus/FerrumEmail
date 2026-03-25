//! Table row component for email layouts.

use ferrum_email_core::*;

/// A table row for multi-column email layouts.
///
/// Contains `Column` components as children. Renders as a `<table>` with
/// a single `<tr>` containing `<td>` cells.
pub struct Row {
    pub children: Vec<Node>,
}

impl Row {
    /// Create a new Row.
    pub fn new() -> Self {
        Row {
            children: Vec::new(),
        }
    }

    /// Add a child component (typically a Column).
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

impl Default for Row {
    fn default() -> Self {
        Row::new()
    }
}

impl Component for Row {
    fn render(&self) -> Node {
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
                .child(Node::Element(Element::new(Tag::Tbody).child(
                    Node::Element(Element::new(Tag::Tr).children(self.children.clone())),
                ))),
        )
    }
}
