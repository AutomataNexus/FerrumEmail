//! Password Reset Email Example
//!
//! Demonstrates a password reset email with security-conscious messaging.
//! Run with: `cargo run -p example-password-reset`

use ferrum_email_components::*;
use ferrum_email_core::Component;
use ferrum_email_send::{providers::ConsoleProvider, Sender};

struct PasswordResetEmail {
    name: String,
    reset_url: String,
    expiry_minutes: u32,
    ip_address: String,
    user_agent: String,
}

impl Component for PasswordResetEmail {
    fn subject(&self) -> Option<&str> {
        Some("Reset your password")
    }

    fn render(&self) -> Node {
        Html::new()
            .child(Head::new().title("Password Reset"))
            .child(Preview::new("You requested a password reset. Click the link below."))
            .child(
                Body::new()
                    .background(Color::hex("f4f4f5"))
                    .child(
                        Container::new()
                            .max_width(Px(600))
                            .background(Color::white())
                            .padding(Spacing::all(Px(0)))
                            .child(
                                Section::new()
                                    .padding(Spacing::new(Px(40), Px(40), Px(0), Px(40)))
                                    .child_node(
                                        Heading::h2("Password Reset Request")
                                            .color(Color::hex("1a1a1a"))
                                            .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                                            .into_node(),
                                    )
                                    .child_node(
                                        Text::new(&format!("Hi {},", self.name))
                                            .color(Color::hex("374151"))
                                            .font_size(Px(16))
                                            .line_height(1.6)
                                            .into_node(),
                                    )
                                    .child_node(
                                        Text::new(
                                            "We received a request to reset your password. \
                                             Click the button below to choose a new password.",
                                        )
                                        .color(Color::hex("374151"))
                                        .font_size(Px(16))
                                        .line_height(1.6)
                                        .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(24)).into_node())
                                    .child_node(
                                        Button::new(&self.reset_url, "Reset Password")
                                            .background(Color::hex("2563eb"))
                                            .text_color(Color::white())
                                            .border_radius(Px(8))
                                            .font_size(Px(16))
                                            .padding(Spacing::xy(Px(14), Px(32)))
                                            .into_node(),
                                    )
                                    .child_node(Spacer::new(Px(24)).into_node())
                                    .child_node(
                                        Text::new(&format!(
                                            "This link will expire in {} minutes.",
                                            self.expiry_minutes
                                        ))
                                        .color(Color::hex("6b7280"))
                                        .font_size(Px(14))
                                        .into_node(),
                                    ),
                            )
                            .child_node(
                                Section::new()
                                    .padding(Spacing::new(Px(24), Px(40), Px(40), Px(40)))
                                    .background(Color::hex("fef3c7"))
                                    .child_node(
                                        Text::new("Security Information")
                                            .font_size(Px(14))
                                            .font_weight(FontWeight::Bold)
                                            .color(Color::hex("92400e"))
                                            .margin(Spacing::new(Px(0), Px(0), Px(8), Px(0)))
                                            .into_node(),
                                    )
                                    .child_node(
                                        Text::new(&format!(
                                            "This request was made from IP address {} using {}.",
                                            self.ip_address, self.user_agent
                                        ))
                                        .font_size(Px(13))
                                        .color(Color::hex("92400e"))
                                        .line_height(1.5)
                                        .into_node(),
                                    )
                                    .child_node(
                                        Text::new(
                                            "If you did not request this password reset, \
                                             please ignore this email. Your password will remain unchanged.",
                                        )
                                        .font_size(Px(13))
                                        .color(Color::hex("92400e"))
                                        .line_height(1.5)
                                        .into_node(),
                                    )
                                    .into_node(),
                            ),
                    ),
            )
            .into_node()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sender = Sender::new(
        ConsoleProvider::new().full_html(),
        "Security <security@automatanexus.com>",
    );

    let email = PasswordResetEmail {
        name: "Andrew".into(),
        reset_url: "https://automatanexus.com/reset/token-xyz-789".into(),
        expiry_minutes: 30,
        ip_address: "192.168.1.100".into(),
        user_agent: "Chrome 120 on macOS".into(),
    };

    println!("Sending password reset email...\n");
    let result = sender.send(&email, "andrew@example.com").await?;
    println!("Sent! Message ID: {}", result.message_id);

    Ok(())
}
