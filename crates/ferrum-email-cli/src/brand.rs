//! brand — the AutomataNexus house template: every mail the agents send goes out branded, never bare HTML.
//!
//! Copyright (c) 2026 Andrew Jewell Sr. / AutomataNexus LLC
//! PROPRIETARY AND CONFIDENTIAL.

use ferrum_email_components::*;
use ferrum_email_core::{Color, Component, Node};
use ferrum_email_core::spacing::Spacing;
use ferrum_email_core::types::Px;
use ferrum_email_render::Renderer;

// ── palette (from the AutomataNexus web properties) ──
const INK: &str = "0e0f10";
const PAPER: &str = "f3efe4";
const TEAL: &str = "14b8a6";
const STONE: &str = "8a8a86";
const LINE: &str = "d0cdc6";
const LOGO: &str = "https://automatanexus.com/logo.png";

pub struct AutomataNexusMail {
    pub subject: String,
    pub body: String,
}

impl Component for AutomataNexusMail {
    fn subject(&self) -> Option<&str> {
        Some(&self.subject)
    }
    fn render(&self) -> Node {
        let mut content = Section::new().background(Color::white()).padding(Spacing::new(Px(32), Px(40), Px(8), Px(40)));
        content = content.child(Heading::h2(&self.subject).color(Color::hex(INK)));
        for para in self.body.split("\n\n") {
            let p = para.trim();
            if p.is_empty() { continue; }
            content = content.child(Text::new(&p.replace('\n', " ")).color(Color::hex(INK)).font_size(Px(15)).line_height(1.6).padding(Spacing::new(Px(8), Px(0), Px(8), Px(0))));
        }
        Html::new()
            .child(Head::new().title(&self.subject))
            .child(Preview::new(self.body.lines().next().unwrap_or(&self.subject)))
            .child(
                Body::new().background(Color::hex(PAPER)).child(
                    Container::new().max_width(Px(600)).background(Color::white()).padding(Spacing::all(Px(0)))
                        .child(Section::new().background(Color::hex(INK)).padding(Spacing::new(Px(24), Px(40), Px(24), Px(40)))
                            .child(Image::new(LOGO, "AutomataNexus", Px(160))))
                        .child(Hr::new().color(Color::hex(TEAL)))
                        .child(content)
                        .child(Section::new().background(Color::white()).padding(Spacing::new(Px(8), Px(40), Px(32), Px(40)))
                            .child(Hr::new().color(Color::hex(LINE)))
                            .child(Text::new("AutomataNexus LLC · automatanexus.com").color(Color::hex(STONE)).font_size(Px(12)))
                            .child(Text::new("Sent by Heka, the AutomataNexus agentic OS, through Ferrum Mail.").color(Color::hex(STONE)).font_size(Px(12)))),
                ),
            )
            .into_node()
    }
}

/// `(html, text)` for the branded mail.
pub fn render(subject: &str, body: &str) -> (String, String) {
    let mail = AutomataNexusMail { subject: subject.to_string(), body: body.to_string() };
    let r = Renderer::default();
    let html = r.render_html(&mail).expect("render html");
    let text = r.render_text(&mail).expect("render text");
    (html, text)
}
