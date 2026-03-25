//! The Node tree — an intermediate representation for email content.
//!
//! Components produce `Node` trees. The renderer walks the tree and emits email-safe HTML.

use crate::style::Style;

/// An HTML tag used in email rendering.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Html,
    Head,
    Meta,
    Title,
    Body,
    Div,
    Span,
    Table,
    Tbody,
    Tr,
    Td,
    Th,
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    A,
    Img,
    Hr,
    Br,
    Pre,
    Code,
    Strong,
    Em,
    /// A custom/raw tag name.
    Custom(String),
}

impl Tag {
    /// Returns the HTML tag name string.
    pub fn as_str(&self) -> &str {
        match self {
            Tag::Html => "html",
            Tag::Head => "head",
            Tag::Meta => "meta",
            Tag::Title => "title",
            Tag::Body => "body",
            Tag::Div => "div",
            Tag::Span => "span",
            Tag::Table => "table",
            Tag::Tbody => "tbody",
            Tag::Tr => "tr",
            Tag::Td => "td",
            Tag::Th => "th",
            Tag::P => "p",
            Tag::H1 => "h1",
            Tag::H2 => "h2",
            Tag::H3 => "h3",
            Tag::H4 => "h4",
            Tag::H5 => "h5",
            Tag::H6 => "h6",
            Tag::A => "a",
            Tag::Img => "img",
            Tag::Hr => "hr",
            Tag::Br => "br",
            Tag::Pre => "pre",
            Tag::Code => "code",
            Tag::Strong => "strong",
            Tag::Em => "em",
            Tag::Custom(name) => name.as_str(),
        }
    }

    /// Returns true if this is a void/self-closing element.
    pub fn is_void(&self) -> bool {
        matches!(self, Tag::Meta | Tag::Img | Tag::Hr | Tag::Br)
    }
}

/// An attribute on an HTML element.
#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: String,
}

impl Attr {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Attr {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// An HTML element in the node tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag: Tag,
    pub attrs: Vec<Attr>,
    pub style: Style,
    pub children: Vec<Node>,
}

impl Element {
    /// Create a new element with the given tag.
    pub fn new(tag: Tag) -> Self {
        Element {
            tag,
            attrs: Vec::new(),
            style: Style::default(),
            children: Vec::new(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push(Attr::new(name, value));
        self
    }

    /// Set the inline style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Add a child node.
    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children.
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }
}

/// A node in the email component tree.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Node {
    /// An HTML element with tag, attributes, style, and children.
    Element(Element),
    /// A text node.
    Text(String),
    /// A fragment containing multiple nodes (no wrapper element).
    Fragment(Vec<Node>),
    /// An empty node (renders nothing).
    None,
}

impl Node {
    /// Create a text node.
    pub fn text(content: impl Into<String>) -> Self {
        Node::Text(content.into())
    }

    /// Create an element node.
    pub fn element(tag: Tag) -> Element {
        Element::new(tag)
    }

    /// Create a fragment from multiple nodes.
    pub fn fragment(nodes: Vec<Node>) -> Self {
        Node::Fragment(nodes)
    }
}
