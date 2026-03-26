//! Built-in email templates for the TUI dashboard.

use ferrum_email_components::*;
use ferrum_email_core::Component;

const FERRUM_LOGO: &str =
    "https://raw.githubusercontent.com/AutomataNexus/FerrumEmail/master/assets/FerrumEmail_logo.PNG";
const NEXUS_LOGO: &str =
    "https://raw.githubusercontent.com/AutomataNexus/FerrumEmail/master/assets/AutomataNexus_Logo.PNG";

/// Template metadata for display in the TUI.
pub struct TemplateMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub subject: &'static str,
}

pub const TEMPLATES: &[TemplateMeta] = &[
    TemplateMeta {
        name: "Welcome Email",
        description: "Onboarding welcome with verify button",
        subject: "Welcome to AutomataNexus",
    },
    TemplateMeta {
        name: "Password Reset",
        description: "Password reset with security info",
        subject: "Reset your password",
    },
    TemplateMeta {
        name: "Security Alert",
        description: "New sign-in detected notification",
        subject: "New sign-in to your account",
    },
    TemplateMeta {
        name: "Test Email",
        description: "Framework test with system info",
        subject: "Ferrum Email — Test Send",
    },
];

/// Render a template by index.
pub fn render_template(index: usize, to_email: &str) -> Box<dyn Component> {
    match index {
        0 => Box::new(WelcomeEmail {
            name: to_email.split('@').next().unwrap_or("User").to_string(),
            verify_url: "https://automatanexus.com/verify/demo".to_string(),
        }),
        1 => Box::new(PasswordResetEmail {
            name: to_email.split('@').next().unwrap_or("User").to_string(),
            reset_url: "https://automatanexus.com/reset/demo".to_string(),
        }),
        2 => Box::new(SecurityAlertEmail {
            name: to_email.split('@').next().unwrap_or("User").to_string(),
            location: "Warsaw, IN".to_string(),
            device: "Chrome on Linux".to_string(),
        }),
        _ => Box::new(TestEmail {
            timestamp: now_string(),
        }),
    }
}

// ── Branded footer (reused across all templates) ──

struct BrandedFooter;

impl Component for BrandedFooter {
    fn render(&self) -> Node {
        Section::new()
            .padding(Spacing::new(Px(28), Px(40), Px(32), Px(40)))
            .background(Color::hex("FAF8F5"))
            .text_align(TextAlign::Center)
            .child_node(Image::new(NEXUS_LOGO, "AutomataNexus", Px(140)).into_node())
            .child_node(Spacer::new(Px(14)).into_node())
            .child_node(
                Text::new("Secured by NexusVault")
                    .color(Color::hex("8B6F5E"))
                    .font_size(Px(13))
                    .font_weight(FontWeight::SemiBold)
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(4), Px(0)))
                    .into_node(),
            )
            .child_node(
                Text::new("AES-256-GCM Encrypted Credential Storage")
                    .color(Color::hex("A8998C"))
                    .font_size(Px(11))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                    .into_node(),
            )
            .child_node(Hr::new().color(Color::hex("E8DDD4")).into_node())
            .child_node(Spacer::new(Px(12)).into_node())
            .child_node(
                Text::new("\u{00A9} 2026 AutomataNexus LLC. All rights reserved.")
                    .color(Color::hex("A8998C"))
                    .font_size(Px(11))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(4), Px(0)))
                    .into_node(),
            )
            .child_node(
                Link::new("https://automatanexus.com", "automatanexus.com")
                    .color(Color::hex("C0582B"))
                    .font_size(Px(11))
                    .into_node(),
            )
            .into_node()
    }
}

fn email_shell(header_child: Node, body_children: Vec<Node>, subject: &str) -> Node {
    Html::new()
        .child(Head::new().title(subject))
        .child(
            Body::new().background(Color::hex("FAFAF8")).child(
                Container::new()
                    .max_width(Px(600))
                    .padding(Spacing::xy(Px(20), Px(0)))
                    .child_node(
                        Section::new()
                            .padding(Spacing::new(Px(40), Px(40), Px(20), Px(40)))
                            .background(Color::hex("FFFEFA"))
                            .text_align(TextAlign::Center)
                            .child_node(header_child)
                            .into_node(),
                    )
                    .child_node(
                        Section::new()
                            .padding(Spacing::new(Px(0), Px(40), Px(32), Px(40)))
                            .background(Color::hex("FFFEFA"))
                            .children(body_children)
                            .into_node(),
                    )
                    .child_node(BrandedFooter.render()),
            ),
        )
        .into_node()
}

// ── Templates ──

pub struct WelcomeEmail {
    pub name: String,
    pub verify_url: String,
}

impl Component for WelcomeEmail {
    fn subject(&self) -> Option<&str> {
        Some("Welcome to AutomataNexus")
    }

    fn render(&self) -> Node {
        email_shell(
            Image::new(FERRUM_LOGO, "Ferrum Email", Px(280)).into_node(),
            vec![
                Heading::h2(&format!("Welcome, {}!", self.name))
                    .color(Color::hex("2D2A26"))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                    .into_node(),
                Text::new(
                    "Thanks for joining AutomataNexus. Click below to verify your email address and get started.",
                )
                .color(Color::hex("4A4540"))
                .font_size(Px(15))
                .line_height(1.6)
                .into_node(),
                Spacer::new(Px(24)).into_node(),
                Button::new(&self.verify_url, "Verify Email Address")
                    .background(Color::hex("C0582B"))
                    .text_color(Color::hex("FFFEFA"))
                    .border_radius(Px(6))
                    .font_size(Px(15))
                    .padding(Spacing::xy(Px(14), Px(28)))
                    .into_node(),
                Spacer::new(Px(24)).into_node(),
                Text::new("If you didn't sign up, you can safely ignore this email.")
                    .color(Color::hex("A8998C"))
                    .font_size(Px(13))
                    .into_node(),
            ],
            "Welcome to AutomataNexus",
        )
    }
}

pub struct PasswordResetEmail {
    pub name: String,
    pub reset_url: String,
}

impl Component for PasswordResetEmail {
    fn subject(&self) -> Option<&str> {
        Some("Reset your password")
    }

    fn render(&self) -> Node {
        email_shell(
            Image::new(FERRUM_LOGO, "Ferrum Email", Px(280)).into_node(),
            vec![
                Heading::h2("Password Reset")
                    .color(Color::hex("2D2A26"))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                    .into_node(),
                Text::new(&format!(
                    "Hi {}, we received a request to reset your password. Click below to choose a new one.",
                    self.name
                ))
                .color(Color::hex("4A4540"))
                .font_size(Px(15))
                .line_height(1.6)
                .into_node(),
                Spacer::new(Px(24)).into_node(),
                Button::new(&self.reset_url, "Reset Password")
                    .background(Color::hex("C0582B"))
                    .text_color(Color::hex("FFFEFA"))
                    .border_radius(Px(6))
                    .font_size(Px(15))
                    .padding(Spacing::xy(Px(14), Px(28)))
                    .into_node(),
                Spacer::new(Px(24)).into_node(),
                Text::new("This link expires in 30 minutes. If you didn't request this, ignore this email.")
                    .color(Color::hex("A8998C"))
                    .font_size(Px(13))
                    .into_node(),
            ],
            "Reset your password",
        )
    }
}

pub struct SecurityAlertEmail {
    pub name: String,
    pub location: String,
    pub device: String,
}

impl Component for SecurityAlertEmail {
    fn subject(&self) -> Option<&str> {
        Some("New sign-in to your account")
    }

    fn render(&self) -> Node {
        email_shell(
            Image::new(FERRUM_LOGO, "Ferrum Email", Px(280)).into_node(),
            vec![
                Heading::h2("New Sign-In Detected")
                    .color(Color::hex("2D2A26"))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                    .into_node(),
                Text::new(&format!(
                    "Hi {}, a new sign-in to your account was detected.",
                    self.name
                ))
                .color(Color::hex("4A4540"))
                .font_size(Px(15))
                .line_height(1.6)
                .into_node(),
                Spacer::new(Px(16)).into_node(),
                Hr::new().color(Color::hex("E8DDD4")).into_node(),
                Spacer::new(Px(12)).into_node(),
                info_row("Location", &self.location),
                info_row("Device", &self.device),
                info_row("Time", &now_string()),
                Spacer::new(Px(20)).into_node(),
                Text::new("If this wasn't you, secure your account immediately.")
                    .color(Color::hex("B4463C"))
                    .font_size(Px(14))
                    .font_weight(FontWeight::SemiBold)
                    .into_node(),
            ],
            "New sign-in to your account",
        )
    }
}

pub struct TestEmail {
    pub timestamp: String,
}

impl Component for TestEmail {
    fn subject(&self) -> Option<&str> {
        Some("Ferrum Email \u{2014} Test Send")
    }

    fn render(&self) -> Node {
        email_shell(
            Image::new(FERRUM_LOGO, "Ferrum Email", Px(280)).into_node(),
            vec![
                Heading::h2("Test Email Sent Successfully")
                    .color(Color::hex("2D2A26"))
                    .text_align(TextAlign::Center)
                    .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                    .into_node(),
                Text::new(
                    "Sent via Ferrum Email's native SMTP provider with STARTTLS. \
                     Credentials loaded from NexusVault (AES-256-GCM).",
                )
                .color(Color::hex("4A4540"))
                .font_size(Px(15))
                .line_height(1.6)
                .into_node(),
                Spacer::new(Px(16)).into_node(),
                Hr::new().color(Color::hex("E8DDD4")).into_node(),
                Spacer::new(Px(12)).into_node(),
                info_row("Framework", "Ferrum Email v0.1.0"),
                info_row("Provider", "Native SMTP + STARTTLS"),
                info_row("Credentials", "NexusVault (AES-256-GCM)"),
                info_row("Sent at", &self.timestamp),
                Spacer::new(Px(24)).into_node(),
                Button::new(
                    "https://github.com/AutomataNexus/FerrumEmail",
                    "View on GitHub",
                )
                .background(Color::hex("C0582B"))
                .text_color(Color::hex("FFFEFA"))
                .border_radius(Px(6))
                .font_size(Px(14))
                .padding(Spacing::xy(Px(14), Px(28)))
                .into_node(),
            ],
            "Ferrum Email \u{2014} Test Send",
        )
    }
}

fn info_row(label: &str, value: &str) -> Node {
    Row::new()
        .child(
            Column::new()
                .width_percent(40.0)
                .padding(Spacing::xy(Px(4), Px(0)))
                .child_node(
                    Text::new(label)
                        .font_weight(FontWeight::SemiBold)
                        .color(Color::hex("8B6F5E"))
                        .font_size(Px(13))
                        .margin(Spacing::xy(Px(2), Px(0)))
                        .into_node(),
                ),
        )
        .child(
            Column::new()
                .width_percent(60.0)
                .padding(Spacing::xy(Px(4), Px(0)))
                .child_node(
                    Text::new(value)
                        .color(Color::hex("2D2A26"))
                        .font_size(Px(13))
                        .margin(Spacing::xy(Px(2), Px(0)))
                        .into_node(),
                ),
        )
        .into_node()
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let t = secs % 86400;
    let (y, mo, d) = days_to_date(days);
    format!(
        "{y:04}-{mo:02}-{d:02} {:02}:{:02}:{:02} UTC",
        t / 3600,
        (t % 3600) / 60,
        t % 60
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    let mut r = days;
    loop {
        let diy = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
            366
        } else {
            365
        };
        if r < diy {
            break;
        }
        r -= diy;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mdays = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0u64;
    for &md in &mdays {
        if r < md {
            break;
        }
        r -= md;
        m += 1;
    }
    (y, m + 1, r + 1)
}
