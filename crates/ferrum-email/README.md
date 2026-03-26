# ferrum-email

**The complete email framework for Rust.** One crate, everything included.

Type-safe templates, cross-client rendering, native SMTP sending, NexusShield security.

## Install

```toml
[dependencies]
ferrum-email = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Send Your First Email

```rust
use ferrum_email::prelude::*;

struct WelcomeEmail { name: String }

impl Component for WelcomeEmail {
    fn subject(&self) -> Option<&str> { Some("Welcome!") }

    fn render(&self) -> Node {
        Html::new()
            .child(Body::new().child(
                Container::new().child(
                    Section::new().padding(Spacing::all(Px(32)))
                        .child_node(
                            Heading::h1(&format!("Hello, {}!", self.name))
                                .color(Color::hex("2D2A26")).into_node())
                        .child_node(
                            Button::new("https://example.com", "Get Started")
                                .background(Color::hex("C0582B"))
                                .text_color(Color::white()).into_node())
                )))
            .into_node()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Send via Ferrum Mail platform (https://ferrum-mail.com)
    let provider = SmtpProvider::builder()
        .host("ferrum-mail.com")
        .port(587)
        .credentials("fm_your_api_key", "")
        .build()?;

    let sender = Sender::new(provider, "you@yourapp.com");
    sender.send(&WelcomeEmail { name: "World".into() }, "user@example.com").await?;
    Ok(())
}
```

## What's Included

| Module | Description |
|--------|-------------|
| `ferrum_email::core` | Component trait, Node tree, Style system, types |
| `ferrum_email::components` | Html, Body, Button, Text, Heading, Image, etc. |
| `ferrum_email::render` | HTML renderer, CSS inliner, plain text extractor |
| `ferrum_email::send` | Sender, SMTP provider, ConsoleProvider |
| `ferrum_email::prelude` | Everything in one `use` statement |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `smtp` | Yes | Native SMTP with STARTTLS |
| `shield` | Yes | NexusShield security validation |
| `vault` | No | NexusVault encrypted credentials |

## Links

- [Ferrum Mail Platform](https://ferrum-mail.com) — Managed email delivery SaaS
- [GitHub](https://github.com/AutomataNexus/FerrumEmail) — Source code
- [docs.rs](https://docs.rs/ferrum-email) — API documentation

## License

MIT OR Apache-2.0
