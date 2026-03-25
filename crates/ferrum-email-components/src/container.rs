//! Centered max-width container using table-based layout.

use ferrum_email_core::*;

/// A centered, max-width container for email content.
///
/// Renders as a table-based layout for maximum email client compatibility.
/// Centers content horizontally and constrains it to a max width.
pub struct Container {
    pub max_width: Px,
    pub background_color: Option<Color>,
    pub padding: Option<Spacing>,
    pub children: Vec<Node>,
}

impl Container {
    /// Create a new Container with a default max-width of 600px.
    pub fn new() -> Self {
        Container {
            max_width: Px(600),
            background_color: None,
            padding: None,
            children: Vec::new(),
        }
    }

    /// Set the max-width.
    pub fn max_width(mut self, width: Px) -> Self {
        self.max_width = width;
        self
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

impl Default for Container {
    fn default() -> Self {
        Container::new()
    }
}

impl Component for Container {
    fn render(&self) -> Node {
        let mut td_style = Style::new();
        td_style.padding = self.padding;

        let mut table_style = Style::new();
        table_style.max_width = Some(SizeValue::Px(self.max_width));
        table_style.width = Some(SizeValue::Percent(Percent(100.0)));
        table_style.background_color = self.background_color.clone();

        // Table-based centering: <table align="center" ...><tbody><tr><td>...children...</td></tr></tbody></table>
        Node::Element(
            Element::new(Tag::Table)
                .attr("align", "center")
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
                                .children(self.children.clone()),
                        )),
                    )),
                )),
        )
    }
}
