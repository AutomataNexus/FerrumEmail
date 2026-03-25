//! Welcome Email Example
//!
//! Demonstrates a complete welcome email template with Ferrum Email.
//! Run with: `cargo run -p example-welcome`

use ferrum_email_components::*;
use ferrum_email_core::Component;
use ferrum_email_send::{providers::ConsoleProvider, Sender};

// ---------------------------------------------------------------------------
// Reusable footer component
// ---------------------------------------------------------------------------

struct EmailFooter {
    company: String,
    address: String,
    unsubscribe_url: String,
}

impl Component for EmailFooter {
    fn render(&self) -> Node {
        Section::new()
            .background(Color::hex("f0f0f0"))
            .padding(Spacing::all(Px(20)))
            .child_node(Hr::new().color(Color::hex("dddddd")).into_node())
            .child_node(
                Text::new(&self.company)
                    .font_size(Px(12))
                    .color(Color::hex("aaaaaa"))
                    .text_align(TextAlign::Center)
                    .into_node(),
            )
            .child_node(
                Text::new(&self.address)
                    .font_size(Px(12))
                    .color(Color::hex("aaaaaa"))
                    .text_align(TextAlign::Center)
                    .into_node(),
            )
            .child_node(
                Link::new(&self.unsubscribe_url, "Unsubscribe")
                    .font_size(Px(12))
                    .color(Color::hex("aaaaaa"))
                    .into_node(),
            )
            .into_node()
    }
}

// ---------------------------------------------------------------------------
// Welcome email template
// ---------------------------------------------------------------------------

struct WelcomeEmail {
    name: String,
    verify_url: String,
    app_name: String,
    unsubscribe_url: String,
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
                            .background(Color::white())
                            .child(
                                Section::new()
                                    .padding(Spacing::all(Px(40)))
                                    .child_node(
                                        Image::new(
                                            "https://automatanexus.com/logo.png",
                                            "AutomataNexus",
                                            Px(150),
                                        )
                                        .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(24)).into_node())
                                    .child_node(
                                        Heading::h1(&format!("Welcome, {}!", self.name))
                                            .color(Color::hex("1a1a1a"))
                                            .into_node(),
                                    )
                                    .child_node(
                                        Text::new(&format!(
                                            "Thanks for signing up for {}. We're excited to have you on board. \
                                             Click the button below to verify your email address and get started.",
                                            self.app_name
                                        ))
                                        .color(Color::hex("555555"))
                                        .font_size(Px(16))
                                        .line_height(1.6)
                                        .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(32)).into_node())
                                    .child_node(
                                        Button::new(&self.verify_url, "Verify Email Address")
                                            .background(Color::hex("C0392B"))
                                            .text_color(Color::white())
                                            .border_radius(Px(6))
                                            .font_size(Px(16))
                                            .padding(Spacing::xy(Px(14), Px(28)))
                                            .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(32)).into_node())
                                    .child_node(
                                        Text::new(
                                            "If you didn't sign up for this account, you can safely ignore this email.",
                                        )
                                        .color(Color::hex("999999"))
                                        .font_size(Px(13))
                                        .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(16)).into_node())
                                    .child_node(
                                        Text::new("Or copy and paste this URL into your browser:")
                                            .color(Color::hex("999999"))
                                            .font_size(Px(12))
                                            .into_node(),
                                    )
                                    .child_node(
                                        Code::new(&self.verify_url)
                                            .font_size(Px(11))
                                            .into_node(),
                                    ),
                            )
                            .child_node(
                                EmailFooter {
                                    company: "AutomataNexus LLC".into(),
                                    address: "Warsaw, IN".into(),
                                    unsubscribe_url: self.unsubscribe_url.clone(),
                                }
                                .render(),
                            ),
                    ),
            )
            .into_node()
    }
}

// ---------------------------------------------------------------------------
// Main — send the email via ConsoleProvider
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sender = Sender::new(
        ConsoleProvider::new().full_html(),
        "AutomataNexus <no-reply@automatanexus.com>",
    );

    let email = WelcomeEmail {
        name: "Andrew".into(),
        verify_url: "https://automatanexus.com/verify/abc123".into(),
        app_name: "AutomataNexus".into(),
        unsubscribe_url: "https://automatanexus.com/unsubscribe/abc123".into(),
    };

    println!("Sending welcome email...\n");
    let result = sender.send(&email, "andrew@example.com").await?;
    println!("Sent! Message ID: {}", result.message_id);

    // Also demonstrate rendering to plain text
    let renderer = ferrum_email_render::Renderer::default();
    let plain_text = renderer.render_text(&email)?;
    println!("\n--- Plain Text Version ---\n{plain_text}");

    Ok(())
}
