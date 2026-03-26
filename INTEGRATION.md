# Ferrum Email — Integration Guide

## Table of Contents

1. [Installation](#installation)
2. [Your First Email](#your-first-email)
3. [Using the Ferrum Mail Platform](#using-the-ferrum-mail-platform)
4. [Self-Hosted SMTP](#self-hosted-smtp)
5. [Building Templates](#building-templates)
6. [Components Reference](#components-reference)
7. [Composing Components](#composing-components)
8. [Rendering Without Sending](#rendering-without-sending)
9. [Plain Text Fallback](#plain-text-fallback)
10. [Security with NexusShield](#security-with-nexusshield)
11. [Credential Storage with NexusVault](#credential-storage-with-nexusvault)
12. [REST API Integration](#rest-api-integration)
13. [Error Handling](#error-handling)
14. [Testing](#testing)

---

## Installation

### Option A: Complete SDK (recommended)

```toml
[dependencies]
ferrum-email = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Option B: Individual crates

```toml
[dependencies]
ferrum-email-core = "0.1"
ferrum-email-components = "0.1"
ferrum-email-render = "0.1"
ferrum-email-send = { version = "0.1", features = ["smtp"] }
```

---

## Your First Email

```rust
use ferrum_email::prelude::*;

struct HelloEmail { name: String }

impl Component for HelloEmail {
    fn subject(&self) -> Option<&str> { Some("Hello!") }

    fn render(&self) -> Node {
        Html::new()
            .child(Body::new().child(
                Container::new().child(
                    Text::new(&format!("Hello, {}!", self.name))
                        .font_size(Px(16))
                )))
            .into_node()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Print to console (no SMTP needed)
    let sender = Sender::new(ConsoleProvider::new(), "me@example.com");
    sender.send(&HelloEmail { name: "World".into() }, "user@example.com").await?;
    Ok(())
}
```

---

## Using the Ferrum Mail Platform

Sign up at [ferrum-mail.com](https://ferrum-mail.com) to get an API key.

### Via Rust SDK

```rust
use ferrum_email::prelude::*;

let provider = SmtpProvider::builder()
    .host("ferrum-mail.com")
    .port(587)
    .credentials("fm_your_api_key", "")
    .build()?;

let sender = Sender::new(provider, "you@yourapp.com");
sender.send(&my_template, "recipient@example.com").await?;
```

### Via REST API

```bash
curl -X POST https://ferrum-mail.com/v1/emails \
  -H "Authorization: fm_your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "from": "you@yourapp.com",
    "to": ["recipient@example.com"],
    "subject": "Hello",
    "html": "<h1>Welcome!</h1>"
  }'
```

### Via Python

```python
import requests

requests.post("https://ferrum-mail.com/v1/emails",
    headers={"Authorization": "fm_your_api_key"},
    json={
        "from": "you@yourapp.com",
        "to": ["recipient@example.com"],
        "subject": "Hello",
        "html": "<h1>Welcome!</h1>",
    })
```

---

## Self-Hosted SMTP

Use any SMTP server — Office365, Gmail, Amazon SES, your own Postfix:

```rust
let provider = SmtpProvider::builder()
    .host("smtp.office365.com")   // or smtp.gmail.com, etc.
    .port(587)
    .credentials("user@example.com", "password")
    .build()?;
```

---

## Building Templates

Templates are Rust structs that implement `Component`:

```rust
use ferrum_email::prelude::*;

pub struct InvoiceEmail {
    pub customer_name: String,
    pub amount: String,
    pub invoice_url: String,
}

impl Component for InvoiceEmail {
    fn subject(&self) -> Option<&str> {
        Some("Your invoice is ready")
    }

    fn render(&self) -> Node {
        Html::new()
            .child(Head::new())
            .child(Body::new().background(Color::hex("FAFAF8")).child(
                Container::new().max_width(Px(600)).child(
                    Section::new().padding(Spacing::all(Px(40)))
                        .child_node(
                            Heading::h2(&format!("Invoice for {}", self.customer_name))
                                .color(Color::hex("2D2A26")).into_node())
                        .child_node(
                            Text::new(&format!("Amount: {}", self.amount))
                                .font_size(Px(18))
                                .font_weight(FontWeight::Bold)
                                .color(Color::hex("C0582B")).into_node())
                        .child_node(Spacer::new(Px(24)).into_node())
                        .child_node(
                            Button::new(&self.invoice_url, "View Invoice")
                                .background(Color::hex("C0582B"))
                                .text_color(Color::white())
                                .border_radius(Px(6)).into_node())
                )))
            .into_node()
    }
}
```

Props are typed at compile time. No `{{ variable }}` strings. No runtime template errors.

---

## Components Reference

| Component | Purpose | Key Props |
|-----------|---------|-----------|
| `Html` | Root element | `lang`, `dir` |
| `Head` | Document head | `title` |
| `Body` | Document body | `background`, `font_family` |
| `Preview` | Inbox preview text | text content |
| `Container` | Centered wrapper | `max_width`, `background` |
| `Section` | Full-width section | `padding`, `background`, `text_align` |
| `Row` | Multi-column row | children |
| `Column` | Table column | `width`, `vertical_align` |
| `Text` | Paragraph | `color`, `font_size`, `line_height`, `font_weight` |
| `Heading` | H1-H6 | `level`, `color`, `font_size` |
| `Button` | CTA button | `href`, `label`, `background`, `text_color`, `border_radius` |
| `Link` | Anchor | `href`, `text`, `color` |
| `Image` | Image | `src`, `alt`, `width`, `height` |
| `Hr` | Divider | `color`, `width` |
| `Code` | Inline code | `content`, `background` |
| `CodeBlock` | Code block | `content`, `font_size` |
| `Spacer` | Vertical space | `height` |

All components use a builder pattern: `Component::new().prop(value).prop(value).into_node()`.

---

## Composing Components

Components compose naturally — call `.render()` on sub-components:

```rust
struct EmailFooter { company: String }

impl Component for EmailFooter {
    fn render(&self) -> Node {
        Section::new()
            .background(Color::hex("FAF8F5"))
            .padding(Spacing::all(Px(20)))
            .child_node(
                Text::new(&format!("(c) 2026 {}", self.company))
                    .font_size(Px(12))
                    .color(Color::hex("A8998C"))
                    .text_align(TextAlign::Center).into_node())
            .into_node()
    }
}

// Use in any template:
impl Component for MyEmail {
    fn render(&self) -> Node {
        Html::new()
            .child(Body::new()
                .child(Container::new()
                    .child_node(/* main content */)
                    .child_node(EmailFooter { company: "Acme Inc".into() }.render())))
            .into_node()
    }
}
```

---

## Rendering Without Sending

Render HTML/text without sending — useful for previews, testing, or piping to another system:

```rust
use ferrum_email::prelude::*;

let renderer = Renderer::default();
let email = MyEmail { /* ... */ };

let html = renderer.render_html(&email)?;   // Full HTML document
let text = renderer.render_text(&email)?;   // Plain text fallback

// Pretty-printed HTML
let renderer = Renderer::with_config(RenderConfig {
    include_doctype: true,
    pretty_print: true,
    indent: "  ".to_string(),
});
let pretty = renderer.render_html(&email)?;
```

---

## Plain Text Fallback

Plain text is auto-generated from the Node tree:

- Links become `text (url)`
- Images become `[alt text]`
- Headings get newlines
- `<hr>` becomes `---`
- Hidden elements (display:none) are stripped

Override auto-generation for custom plain text:

```rust
impl Component for MyEmail {
    fn plain_text(&self) -> Option<String> {
        Some(format!("Hello {}! Visit https://example.com", self.name))
    }
    fn render(&self) -> Node { /* ... */ }
}
```

---

## Security with NexusShield

When the `shield` feature is enabled (default), every outbound email is validated:

- **Address validation** — format, blocked domains, disposable addresses
- **Header injection** — CRLF in subject, names, addresses
- **Rate limiting** — per-recipient bombing prevention
- **Encoded attacks** — base64 payloads, unicode tricks

This runs automatically. No configuration needed.

---

## Credential Storage with NexusVault

Enable the `vault` feature for encrypted credential storage:

```toml
ferrum-email = { version = "0.1", features = ["vault"] }
```

```rust
use ferrum_email::send::vault::VaultCredentialStore;

let store = VaultCredentialStore::new(vault);
store.set_smtp_credentials("user", "pass", "smtp.example.com", 587)?;
store.set_api_key("fm_your_key")?;

// Credentials encrypted with AES-256-GCM at rest
let password = store.get_smtp_password()?;
```

---

## REST API Integration

The Ferrum Mail platform at [ferrum-mail.com](https://ferrum-mail.com) provides a REST API:

### Send Email
```
POST /v1/emails
Authorization: fm_your_api_key

{
  "from": "you@yourapp.com",
  "to": ["recipient@example.com"],
  "subject": "Hello",
  "html": "<h1>Welcome!</h1>",
  "text": "Welcome!"           // optional plain text
}
```

### Response
```json
{
  "id": "uuid",
  "message_id": "smtp-message-id"
}
```

### Errors
```json
{
  "error": "Monthly quota exceeded (100/100). Upgrade your plan."
}
```

### Plans

| Plan | Price | Emails/Month |
|------|-------|-------------|
| Free | $0 | 100 |
| Developer | $20 | 10,000 |
| Business | $100 | 100,000 |
| Enterprise | $899 | Unlimited |

---

## Error Handling

```rust
match sender.send(&email, "user@example.com").await {
    Ok(result) => println!("Sent: {}", result.message_id),
    Err(EmailError::Render(e)) => eprintln!("Template error: {e}"),
    Err(EmailError::Provider(e)) => eprintln!("SMTP error: {e}"),
    Err(EmailError::InvalidAddress(e)) => eprintln!("Bad address: {e}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

---

## Testing

Templates are pure Rust structs — test them like any other code:

```rust
#[test]
fn welcome_email_renders() {
    let email = WelcomeEmail { name: "Test".into(), /* ... */ };
    let renderer = Renderer::default();
    let html = renderer.render_html(&email).unwrap();

    assert!(html.contains("Test"));
    assert!(html.contains("<!DOCTYPE html>"));
}

#[test]
fn welcome_email_subject() {
    let email = WelcomeEmail { name: "Test".into(), /* ... */ };
    assert_eq!(email.subject(), Some("Welcome!"));
}

#[test]
fn welcome_email_plain_text_clean() {
    let email = WelcomeEmail { name: "Test".into(), /* ... */ };
    let renderer = Renderer::default();
    let text = renderer.render_text(&email).unwrap();
    assert!(!text.contains('<'));
}

#[tokio::test]
async fn send_to_console() {
    let sender = Sender::new(ConsoleProvider::new(), "test@example.com");
    let result = sender.send(&email, "user@example.com").await.unwrap();
    assert_eq!(result.provider, "console");
}
```
