//! CSS inlining for email-safe rendering.
//!
//! Since Gmail and many email clients strip `<style>` blocks from `<head>`,
//! all CSS must be inlined as `style=""` attributes on each element.
//! The Ferrum Style system already produces inline CSS via `Style::to_css()`,
//! so this module handles merging and normalization.

use ferrum_email_core::{Element, Node, Style};

/// Walk a Node tree and ensure all styles are properly inlined.
///
/// This is largely a pass-through since Ferrum's component system already
/// produces inline styles via the `Style` struct. This function handles
/// edge cases like merging inherited styles and normalizing the output.
pub fn inline_styles(node: &Node) -> Node {
    inline_node(node, &Style::default())
}

fn inline_node(node: &Node, _inherited: &Style) -> Node {
    match node {
        Node::Element(element) => {
            let mut new_element = Element::new(element.tag.clone());

            // Check if the element already has a manually-set style attribute.
            // If so, we don't override it — the manually-set attribute takes precedence.
            let has_manual_style = element.attrs.iter().any(|a| a.name == "style");

            // Copy all attributes except style (we'll regenerate it)
            for attr in &element.attrs {
                if attr.name == "style" && !has_manual_style {
                    continue;
                }
                new_element = new_element.attr(&attr.name, &attr.value);
            }

            // If there's no manual style attribute, generate one from the Style struct
            if !has_manual_style {
                new_element.style = element.style.clone();

                // Also set the style as an attribute for the HTML emitter
                if let Some(css) = element.style.to_css() {
                    new_element = new_element.attr("style", css);
                }
            }

            // Recursively process children
            let children: Vec<Node> = element
                .children
                .iter()
                .map(|child| inline_node(child, &element.style))
                .collect();
            new_element = new_element.children(children);

            Node::Element(new_element)
        }
        Node::Fragment(nodes) => {
            let children: Vec<Node> = nodes
                .iter()
                .map(|child| inline_node(child, _inherited))
                .collect();
            Node::Fragment(children)
        }
        other => other.clone(),
    }
}
