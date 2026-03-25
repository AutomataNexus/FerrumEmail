//! Integration tests for the full Ferrum Email pipeline.
//!
//! Tests the complete flow: define a component → render to HTML/text → send via ConsoleProvider.

use ferrum_email_components::*;
use ferrum_email_core::Component;
use ferrum_email_render::Renderer;
use ferrum_email_send::{providers::ConsoleProvider, Mailbox, Sender};

// ---------------------------------------------------------------------------
// Test email templates
// ---------------------------------------------------------------------------

struct WelcomeEmail {
    name: String,
    verify_url: String,
    app_name: String,
}

impl Component for WelcomeEmail {
    fn subject(&self) -> Option<&str> {
        Some("Welcome to AutomataNexus")
    }

    fn render(&self) -> Node {
        Html::new()
            .child(Head::new().title("Welcome"))
            .child(Preview::new(format!(
                "Welcome, {}! Verify your email to get started.",
                self.name
            )))
            .child(
                Body::new()
                    .background(Color::hex("f6f6f6"))
                    .child(
                        Container::new()
                            .max_width(Px(600))
                            .child(
                                Section::new()
                                    .padding(Spacing::all(Px(32)))
                                    .child_node(Heading::h1(&format!("Welcome, {}!", self.name))
                                        .color(Color::hex("1a1a1a"))
                                        .into_node())
                                    .child_node(
                                        Text::new(&format!(
                                            "Thanks for signing up for {}. Click the button below to verify your email address.",
                                            self.app_name
                                        ))
                                        .color(Color::hex("555555"))
                                        .font_size(Px(16))
                                        .line_height(1.6)
                                        .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(24)).into_node())
                                    .child_node(
                                        Button::new(&self.verify_url, "Verify Email Address")
                                            .background(Color::hex("C0392B"))
                                            .text_color(Color::white())
                                            .border_radius(Px(6))
                                            .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(24)).into_node())
                                    .child_node(
                                        Text::new(
                                            "If you didn't sign up for this account, you can safely ignore this email.",
                                        )
                                        .color(Color::hex("999999"))
                                        .font_size(Px(13))
                                        .into_node(),
                                    ),
                            ),
                    ),
            )
            .into_node()
    }
}

struct SimpleEmail;

impl Component for SimpleEmail {
    fn subject(&self) -> Option<&str> {
        Some("Hello!")
    }

    fn render(&self) -> Node {
        Html::new()
            .child(Body::new().child(Text::new("Hello, World!")))
            .into_node()
    }
}

// ---------------------------------------------------------------------------
// HTML rendering tests
// ---------------------------------------------------------------------------

#[test]
fn welcome_email_renders_valid_html() {
    let email = WelcomeEmail {
        name: "Andrew".into(),
        verify_url: "https://example.com/verify/abc123".into(),
        app_name: "AutomataNexus".into(),
    };
    let renderer = Renderer::default();
    let html = renderer.render_html(&email).unwrap();

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));
    assert!(html.contains("Andrew"));
    assert!(html.contains("https://example.com/verify/abc123"));
    assert!(html.contains("Verify Email Address"));
    assert!(html.contains("AutomataNexus"));
}

#[test]
fn welcome_email_has_correct_subject() {
    let email = WelcomeEmail {
        name: "Test".into(),
        verify_url: "https://example.com".into(),
        app_name: "TestApp".into(),
    };
    assert_eq!(email.subject(), Some("Welcome to AutomataNexus"));
}

#[test]
fn welcome_email_plain_text_has_no_html_tags() {
    let email = WelcomeEmail {
        name: "Test User".into(),
        verify_url: "https://example.com/verify/abc123".into(),
        app_name: "TestApp".into(),
    };
    let renderer = Renderer::default();
    let text = renderer.render_text(&email).unwrap();

    assert!(!text.contains('<'));
    assert!(!text.contains('>'));
    assert!(text.contains("Test User"));
}

#[test]
fn css_is_inlined_no_style_blocks() {
    let email = WelcomeEmail {
        name: "Test".into(),
        verify_url: "https://example.com".into(),
        app_name: "TestApp".into(),
    };
    let renderer = Renderer::default();
    let html = renderer.render_html(&email).unwrap();

    // Should have inline style attributes
    assert!(html.contains("style=\""));

    // Should NOT have <style> blocks in the output (Gmail strips them)
    assert!(!html.contains("<style>"));
    assert!(!html.contains("<style "));
}

#[test]
fn button_renders_as_table() {
    let button = Button::new("https://example.com", "Click Me")
        .background(Color::hex("C0392B"))
        .text_color(Color::white());

    let renderer = Renderer::default();
    let html = renderer.render_node(&button.render());

    // Button should use table-based layout for Outlook compatibility
    assert!(html.contains("<table"));
    assert!(html.contains("role=\"presentation\""));
    assert!(html.contains("<td"));
    assert!(html.contains("<a"));
    assert!(html.contains("href=\"https://example.com\""));
    assert!(html.contains("Click Me"));
}

#[test]
fn container_renders_centered_table() {
    let container = Container::new().max_width(Px(600));
    let renderer = Renderer::default();
    let html = renderer.render_node(&container.render());

    assert!(html.contains("align=\"center\""));
    assert!(html.contains("role=\"presentation\""));
    assert!(html.contains("cellpadding=\"0\""));
    assert!(html.contains("cellspacing=\"0\""));
}

#[test]
fn heading_levels_produce_correct_tags() {
    let renderer = Renderer::default();

    let h1 = renderer.render_node(&Heading::h1("Title").render());
    assert!(h1.contains("<h1"));
    assert!(h1.contains("</h1>"));

    let h3 = renderer.render_node(&Heading::h3("Subtitle").render());
    assert!(h3.contains("<h3"));
    assert!(h3.contains("</h3>"));
}

#[test]
fn image_renders_with_required_attributes() {
    let img = Image::new("https://example.com/logo.png", "Logo", Px(200))
        .height(Px(50));
    let renderer = Renderer::default();
    let html = renderer.render_node(&img.render());

    assert!(html.contains("src=\"https://example.com/logo.png\""));
    assert!(html.contains("alt=\"Logo\""));
    assert!(html.contains("width=\"200\""));
    assert!(html.contains("height=\"50\""));
}

#[test]
fn hr_renders_as_self_closing() {
    let hr = Hr::new();
    let renderer = Renderer::default();
    let html = renderer.render_node(&hr.render());

    assert!(html.contains("<hr"));
    assert!(!html.contains("</hr>"));
}

#[test]
fn link_renders_with_href() {
    let link = Link::new("https://example.com", "Example")
        .color(Color::hex("067df7"));
    let renderer = Renderer::default();
    let html = renderer.render_node(&link.render());

    assert!(html.contains("href=\"https://example.com\""));
    assert!(html.contains("Example"));
    assert!(html.contains("target=\"_blank\""));
}

#[test]
fn link_plain_text_includes_url() {
    let link = Link::new("https://example.com", "Click here");
    let text = ferrum_email_render::text_extractor::extract_text(&link.render());

    assert!(text.contains("Click here"));
    assert!(text.contains("https://example.com"));
}

#[test]
fn spacer_renders_with_height() {
    let spacer = Spacer::new(Px(24));
    let renderer = Renderer::default();
    let html = renderer.render_node(&spacer.render());

    assert!(html.contains("height=\"24\""));
}

#[test]
fn code_block_renders_pre_code() {
    let code = CodeBlock::new("fn main() { println!(\"hello\"); }");
    let renderer = Renderer::default();
    let html = renderer.render_node(&code.render());

    assert!(html.contains("<pre"));
    assert!(html.contains("<code"));
    assert!(html.contains("fn main()"));
}

#[test]
fn row_column_layout() {
    let layout = Row::new()
        .child(Column::new().width_percent(50.0).child(Text::new("Left")))
        .child(Column::new().width_percent(50.0).child(Text::new("Right")));

    let renderer = Renderer::default();
    let html = renderer.render_node(&layout.render());

    assert!(html.contains("<table"));
    assert!(html.contains("<td"));
    assert!(html.contains("Left"));
    assert!(html.contains("Right"));
    assert!(html.contains("width=\"50%\""));
}

#[test]
fn preview_text_is_hidden() {
    let preview = Preview::new("Preview text here");
    let renderer = Renderer::default();
    let html = renderer.render_node(&preview.render());

    assert!(html.contains("Preview text here"));
    assert!(html.contains("display:none"));
}

#[test]
fn section_renders_full_width_table() {
    let section = Section::new()
        .background(Color::hex("ffffff"))
        .padding(Spacing::all(Px(20)));
    let renderer = Renderer::default();
    let html = renderer.render_node(&section.render());

    assert!(html.contains("width=\"100%\""));
    assert!(html.contains("<table"));
}

// ---------------------------------------------------------------------------
// Sender integration test (console provider, no network)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_via_console_provider() {
    let sender = Sender::new(
        ConsoleProvider::new(),
        "AutomataNexus <no-reply@automatanexus.com>",
    );

    let email = WelcomeEmail {
        name: "Andrew".into(),
        verify_url: "https://example.com/verify/abc123".into(),
        app_name: "AutomataNexus".into(),
    };

    let result = sender.send(&email, "andrew@example.com").await.unwrap();

    assert_eq!(result.provider, "console");
    assert!(result.message_id.starts_with("console-"));
}

#[tokio::test]
async fn send_batch_via_console_provider() {
    let sender = Sender::new(
        ConsoleProvider::new(),
        "test@example.com",
    );

    let email = SimpleEmail;
    let recipients = vec![
        Mailbox::address("user1@example.com"),
        Mailbox::address("user2@example.com"),
    ];

    let results = sender.send_batch(&email, recipients).await.unwrap();

    assert_eq!(results.len(), 2);
    for result in &results {
        assert_eq!(result.provider, "console");
    }
}

// ---------------------------------------------------------------------------
// Style system tests
// ---------------------------------------------------------------------------

#[test]
fn style_to_css_renders_all_set_properties() {
    let mut style = Style::new();
    style.color = Some(Color::hex("ff0000"));
    style.font_size = Some(Px(16));
    style.font_weight = Some(FontWeight::Bold);
    style.text_align = Some(TextAlign::Center);

    let css = style.to_css().unwrap();

    assert!(css.contains("color:#ff0000"));
    assert!(css.contains("font-size:16px"));
    assert!(css.contains("font-weight:700"));
    assert!(css.contains("text-align:center"));
}

#[test]
fn style_to_css_returns_none_when_empty() {
    let style = Style::new();
    assert!(style.to_css().is_none());
}

#[test]
fn spacing_formatting() {
    assert_eq!(Spacing::all(Px(10)).to_string(), "10px");
    assert_eq!(Spacing::xy(Px(10), Px(20)).to_string(), "10px 20px");
    assert_eq!(
        Spacing::new(Px(1), Px(2), Px(3), Px(4)).to_string(),
        "1px 2px 3px 4px"
    );
}

#[test]
fn color_formatting() {
    assert_eq!(Color::hex("ff0000").to_string(), "#ff0000");
    assert_eq!(Color::rgb(255, 0, 0).to_string(), "rgb(255,0,0)");
    assert_eq!(Color::white().to_string(), "#ffffff");
    assert_eq!(Color::black().to_string(), "#000000");
    assert_eq!(Color::transparent().to_string(), "transparent");
}

#[test]
fn pretty_print_renderer() {
    let config = ferrum_email_render::RenderConfig {
        include_doctype: false,
        pretty_print: true,
        indent: "  ".to_string(),
    };
    let renderer = Renderer::with_config(config);
    let html = renderer
        .render_html(&SimpleEmail)
        .unwrap();

    // Pretty-printed HTML should have newlines and indentation
    assert!(html.contains('\n'));
    assert!(html.lines().count() > 1);
}
