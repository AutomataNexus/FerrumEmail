//! HTML string building utilities — entity escaping and tag emission.

/// Escape text content for safe inclusion in HTML.
///
/// Escapes `&`, `<`, `>` which are the characters that would break HTML parsing.
pub fn escape_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(ch),
        }
    }
    output
}

/// Escape an attribute value for safe inclusion in a quoted HTML attribute.
///
/// Escapes `&`, `<`, `>`, `"`, and `'`.
pub fn escape_attr(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(ch),
        }
    }
    output
}

/// Emit the HTML5 DOCTYPE declaration.
pub fn doctype() -> &'static str {
    "<!DOCTYPE html>"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_text() {
        assert_eq!(escape_text("Hello & World"), "Hello &amp; World");
        assert_eq!(escape_text("<script>"), "&lt;script&gt;");
        assert_eq!(escape_text("plain text"), "plain text");
    }

    #[test]
    fn test_escape_attr() {
        assert_eq!(escape_attr("say \"hello\""), "say &quot;hello&quot;");
        assert_eq!(escape_attr("it's"), "it&#x27;s");
        assert_eq!(escape_attr("a&b"), "a&amp;b");
    }
}
