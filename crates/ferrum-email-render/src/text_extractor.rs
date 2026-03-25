//! Plain text extraction from a Node tree.
//!
//! Walks the component tree and extracts readable plain text,
//! stripping all HTML markup and converting structural elements
//! into text-friendly formatting.

use ferrum_email_core::{Node, Tag};

/// Extract plain text from a Node tree.
///
/// Converts the node tree into a human-readable plain text representation:
/// - Text nodes are included as-is
/// - Block-level elements (p, h1-h6, div, tr) get newlines around them
/// - Links become `text (url)`
/// - Horizontal rules become `---`
/// - Images become `[alt text]`
/// - Fragments and other nodes are recursively processed
pub fn extract_text(node: &Node) -> String {
    let mut output = String::new();
    extract_node(node, &mut output);
    // Clean up excessive whitespace
    clean_text(&output)
}

fn extract_node(node: &Node, output: &mut String) {
    match node {
        Node::Text(text) => {
            output.push_str(text);
        }
        Node::Element(element) => {
            let tag = &element.tag;

            // Handle special elements
            match tag {
                Tag::A => {
                    // Extract link text and href
                    let link_text = extract_children_text(&element.children);
                    let href = element
                        .attrs
                        .iter()
                        .find(|a| a.name == "href")
                        .map(|a| a.value.as_str())
                        .unwrap_or("");

                    if !link_text.is_empty() && !href.is_empty() && link_text != href {
                        output.push_str(&link_text);
                        output.push_str(" (");
                        output.push_str(href);
                        output.push(')');
                    } else if !link_text.is_empty() {
                        output.push_str(&link_text);
                    } else if !href.is_empty() {
                        output.push_str(href);
                    }
                    return;
                }
                Tag::Img => {
                    let alt = element
                        .attrs
                        .iter()
                        .find(|a| a.name == "alt")
                        .map(|a| a.value.as_str())
                        .unwrap_or("");
                    if !alt.is_empty() {
                        output.push('[');
                        output.push_str(alt);
                        output.push(']');
                    }
                    return;
                }
                Tag::Hr => {
                    output.push_str("\n---\n");
                    return;
                }
                Tag::Br => {
                    output.push('\n');
                    return;
                }
                Tag::Head | Tag::Meta | Tag::Title => {
                    // Skip head content in plain text
                    return;
                }
                _ => {}
            }

            // Check if preview text (hidden div) — skip it
            if is_hidden_element(element) {
                return;
            }

            let is_block = is_block_element(tag);

            if is_block {
                output.push('\n');
            }

            for child in &element.children {
                extract_node(child, output);
            }

            if is_block {
                output.push('\n');
            }
        }
        Node::Fragment(nodes) => {
            for node in nodes {
                extract_node(node, output);
            }
        }
        Node::None => {}
    }
}

fn extract_children_text(children: &[Node]) -> String {
    let mut output = String::new();
    for child in children {
        extract_node(child, &mut output);
    }
    output.trim().to_string()
}

fn is_block_element(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::P
            | Tag::Div
            | Tag::H1
            | Tag::H2
            | Tag::H3
            | Tag::H4
            | Tag::H5
            | Tag::H6
            | Tag::Tr
            | Tag::Table
            | Tag::Pre
    )
}

fn is_hidden_element(element: &ferrum_email_core::Element) -> bool {
    // Check for display:none in style
    if let Some(ref display) = element.style.display {
        if *display == ferrum_email_core::Display::None {
            return true;
        }
    }
    // Check for style attribute containing display:none
    element
        .attrs
        .iter()
        .any(|a| a.name == "style" && a.value.contains("display:none"))
}

/// Clean up excessive whitespace in extracted text.
fn clean_text(input: &str) -> String {
    let mut lines: Vec<&str> = input.lines().collect();

    // Trim each line
    let lines: Vec<&str> = lines.iter_mut().map(|l| l.trim()).collect();

    // Remove excessive blank lines (more than 2 consecutive)
    let mut result = String::new();
    let mut blank_count = 0;

    for line in &lines {
        if line.is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_email_core::{Element, Node, Tag};

    #[test]
    fn test_extract_text_from_text_node() {
        let node = Node::text("Hello, World!");
        assert_eq!(extract_text(&node), "Hello, World!");
    }

    #[test]
    fn test_extract_text_from_link() {
        let node = Node::Element(
            Element::new(Tag::A)
                .attr("href", "https://example.com")
                .child(Node::text("Click here")),
        );
        assert_eq!(extract_text(&node), "Click here (https://example.com)");
    }

    #[test]
    fn test_extract_text_from_hr() {
        let node = Node::Element(Element::new(Tag::Hr));
        assert_eq!(extract_text(&node), "---");
    }

    #[test]
    fn test_extract_text_from_image() {
        let node = Node::Element(
            Element::new(Tag::Img)
                .attr("alt", "Logo")
                .attr("src", "logo.png"),
        );
        assert_eq!(extract_text(&node), "[Logo]");
    }
}
