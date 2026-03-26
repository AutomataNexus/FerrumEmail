//! The core renderer — walks a Node tree and emits email-safe HTML.

use ferrum_email_core::{Component, Node};

use crate::RenderError;
use crate::css_inliner;
use crate::html_emitter::{doctype, escape_attr, escape_text};
use crate::text_extractor;

/// Configuration for the renderer.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Whether to prepend `<!DOCTYPE html>` to the output.
    pub include_doctype: bool,
    /// Whether to pretty-print the HTML with indentation.
    pub pretty_print: bool,
    /// Indentation string for pretty-printing (default: "  ").
    pub indent: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        RenderConfig {
            include_doctype: true,
            pretty_print: false,
            indent: "  ".to_string(),
        }
    }
}

/// The Ferrum Email renderer.
///
/// Takes a `Component`, calls its `render()` method to produce a `Node` tree,
/// inlines CSS styles, and emits email-safe HTML.
pub struct Renderer {
    pub config: RenderConfig,
}

impl Renderer {
    /// Create a new renderer with default configuration.
    pub fn new() -> Self {
        Renderer {
            config: RenderConfig::default(),
        }
    }

    /// Create a new renderer with custom configuration.
    pub fn with_config(config: RenderConfig) -> Self {
        Renderer { config }
    }

    /// Render a component to an HTML string.
    pub fn render_html(&self, component: &dyn Component) -> Result<String, RenderError> {
        let node = component.render();
        let inlined = css_inliner::inline_styles(&node);

        let mut output = String::new();
        if self.config.include_doctype {
            output.push_str(doctype());
            output.push('\n');
        }

        if self.config.pretty_print {
            self.emit_node_pretty(&inlined, &mut output, 0);
        } else {
            self.emit_node(&inlined, &mut output);
        }

        Ok(output)
    }

    /// Render a component to a plain text string.
    pub fn render_text(&self, component: &dyn Component) -> Result<String, RenderError> {
        // Check if the component provides custom plain text
        if let Some(custom_text) = component.plain_text() {
            return Ok(custom_text);
        }

        let node = component.render();
        Ok(text_extractor::extract_text(&node))
    }

    /// Render a single Node to an HTML string (without DOCTYPE).
    pub fn render_node(&self, node: &Node) -> String {
        let inlined = css_inliner::inline_styles(node);
        let mut output = String::new();
        self.emit_node(&inlined, &mut output);
        output
    }

    /// Emit a node to the output string (compact mode).
    fn emit_node(&self, node: &Node, output: &mut String) {
        match node {
            Node::Text(text) => {
                output.push_str(&escape_text(text));
            }
            Node::Element(element) => {
                let tag_name = element.tag.as_str();

                // Open tag
                output.push('<');
                output.push_str(tag_name);

                // Attributes
                for attr in &element.attrs {
                    output.push(' ');
                    output.push_str(&attr.name);
                    output.push_str("=\"");
                    output.push_str(&escape_attr(&attr.value));
                    output.push('"');
                }

                if element.tag.is_void() {
                    output.push_str(" />");
                    return;
                }

                output.push('>');

                // Children
                for child in &element.children {
                    self.emit_node(child, output);
                }

                // Close tag
                output.push_str("</");
                output.push_str(tag_name);
                output.push('>');
            }
            Node::Fragment(nodes) => {
                for node in nodes {
                    self.emit_node(node, output);
                }
            }
            Node::None => {}
        }
    }

    /// Emit a node with pretty-printing (indentation and newlines).
    fn emit_node_pretty(&self, node: &Node, output: &mut String, depth: usize) {
        let indent = self.config.indent.repeat(depth);

        match node {
            Node::Text(text) => {
                let escaped = escape_text(text);
                if !escaped.trim().is_empty() {
                    output.push_str(&indent);
                    output.push_str(&escaped);
                    output.push('\n');
                }
            }
            Node::Element(element) => {
                let tag_name = element.tag.as_str();

                // Open tag
                output.push_str(&indent);
                output.push('<');
                output.push_str(tag_name);

                for attr in &element.attrs {
                    output.push(' ');
                    output.push_str(&attr.name);
                    output.push_str("=\"");
                    output.push_str(&escape_attr(&attr.value));
                    output.push('"');
                }

                if element.tag.is_void() {
                    output.push_str(" />\n");
                    return;
                }

                output.push_str(">\n");

                // Children
                for child in &element.children {
                    self.emit_node_pretty(child, output, depth + 1);
                }

                // Close tag
                output.push_str(&indent);
                output.push_str("</");
                output.push_str(tag_name);
                output.push_str(">\n");
            }
            Node::Fragment(nodes) => {
                for node in nodes {
                    self.emit_node_pretty(node, output, depth);
                }
            }
            Node::None => {}
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_email_core::*;

    struct SimpleEmail;

    impl Component for SimpleEmail {
        fn render(&self) -> Node {
            Node::Element(Element::new(Tag::P).child(Node::text("Hello, World!")))
        }
    }

    #[test]
    fn test_render_simple_html() {
        let renderer = Renderer::new();
        let html = renderer.render_html(&SimpleEmail).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<p>Hello, World!</p>"));
    }

    #[test]
    fn test_render_simple_text() {
        let renderer = Renderer::new();
        let text = renderer.render_text(&SimpleEmail).unwrap();
        assert!(text.contains("Hello, World!"));
        assert!(!text.contains('<'));
    }

    #[test]
    fn test_html_escaping() {
        struct EscapeEmail;
        impl Component for EscapeEmail {
            fn render(&self) -> Node {
                Node::Element(Element::new(Tag::P).child(Node::text("1 < 2 & 3 > 2")))
            }
        }
        let renderer = Renderer::new();
        let html = renderer.render_html(&EscapeEmail).unwrap();
        assert!(html.contains("1 &lt; 2 &amp; 3 &gt; 2"));
    }

    #[test]
    fn test_void_elements() {
        struct VoidEmail;
        impl Component for VoidEmail {
            fn render(&self) -> Node {
                Node::Element(
                    Element::new(Tag::Img)
                        .attr("src", "test.png")
                        .attr("alt", "test"),
                )
            }
        }
        let renderer = Renderer::new();
        let html = renderer.render_html(&VoidEmail).unwrap();
        assert!(html.contains("<img src=\"test.png\" alt=\"test\" />"));
        assert!(!html.contains("</img>"));
    }

    #[test]
    fn test_style_inlining() {
        struct StyledEmail;
        impl Component for StyledEmail {
            fn render(&self) -> Node {
                let mut style = Style::new();
                style.color = Some(Color::hex("ff0000"));
                style.font_size = Some(Px(16));

                Node::Element(
                    Element::new(Tag::P)
                        .style(style)
                        .child(Node::text("Red text")),
                )
            }
        }
        let renderer = Renderer::new();
        let html = renderer.render_html(&StyledEmail).unwrap();
        assert!(html.contains("style=\""));
        assert!(html.contains("color:#ff0000"));
        assert!(html.contains("font-size:16px"));
    }
}
