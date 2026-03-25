//! Preview text component — the text shown in email client inbox lists.

use ferrum_email_core::*;

/// Preview text shown in email client inbox previews.
///
/// Uses the invisible overflow trick: the preview text is followed by
/// invisible whitespace characters that prevent the email body content
/// from leaking into the preview line in email clients.
pub struct Preview {
    pub text: String,
}

impl Preview {
    /// Create a new Preview with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Preview { text: text.into() }
    }

    /// Convert this component into a Node.
    pub fn into_node(self) -> Node {
        self.render()
    }
}

impl Component for Preview {
    fn render(&self) -> Node {
        // The preview text is rendered in a hidden div, followed by invisible
        // whitespace characters to prevent email body content from appearing
        // in the preview line of email clients.
        let mut hidden_style = Style::new();
        hidden_style.display = Some(Display::None);
        hidden_style.max_width = Some(SizeValue::Px(Px(0)));
        hidden_style.height = Some(SizeValue::Px(Px(0)));

        // Generate invisible whitespace to fill the preview space
        // Using a mix of zero-width spaces and regular spaces
        let filler = "\u{200C}\u{00A0}\u{200B}\u{200C}\u{00A0}\u{200B}".repeat(30);

        Node::Fragment(vec![
            Node::Element(
                Element::new(Tag::Div)
                    .style({
                        let mut s = Style::new();
                        s.display = Some(Display::None);
                        s.font_size = Some(Px(1));
                        s.color = Some(Color::white());
                        s.line_height = Some(LineHeight::Multiplier(1.0));
                        s.max_width = Some(SizeValue::Px(Px(0)));
                        s.height = Some(SizeValue::Px(Px(0)));
                        s
                    })
                    .attr("style", "display:none;font-size:1px;color:#ffffff;line-height:1;max-height:0;max-width:0;opacity:0;overflow:hidden")
                    .child(Node::text(&self.text)),
            ),
            Node::Element(
                Element::new(Tag::Div)
                    .style(hidden_style)
                    .attr("style", "display:none;max-height:0;overflow:hidden")
                    .child(Node::text(filler)),
            ),
        ])
    }
}
