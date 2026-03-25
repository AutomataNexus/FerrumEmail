# ferrum-email-send

Email provider abstraction and sending API for [Ferrum Email](https://github.com/AutomataNexus/ferrum-email).

## Overview

This crate provides:

- **`Sender`** — the main API. Takes a component, renders it, sends it.
- **`EmailProvider`** trait — implement this for any email backend.
- **`EmailMessage`** — the complete email structure (from, to, subject, html, text, attachments, etc.)
- **`Mailbox`** — email address with optional display name. Parses `"Name <email@domain>"` format.
- **`ConsoleProvider`** — prints emails to stdout for development.

## Quick Start

```rust
use ferrum_email_send::{Sender, providers::ConsoleProvider};

let sender = Sender::new(
    ConsoleProvider::new(),
    "App <no-reply@example.com>",
);

// Send any Component:
sender.send(&my_email_component, "user@example.com").await?;
```

## Sender API

```rust
// Send to one recipient (renders component automatically)
sender.send(&component, "user@example.com").await?;

// Send a pre-built message
sender.send_message(email_message).await?;

// Send to multiple recipients
sender.send_batch(&component, vec![
    Mailbox::address("user1@example.com"),
    Mailbox::address("user2@example.com"),
]).await?;
```

## Custom Providers

```rust
use async_trait::async_trait;
use ferrum_email_send::{EmailProvider, EmailMessage, SendResult, EmailError};

struct MyProvider { api_key: String }

#[async_trait]
impl EmailProvider for MyProvider {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        // Your sending logic here
        Ok(SendResult {
            message_id: "msg-123".into(),
            provider: "my-provider".into(),
        })
    }
}
```

## Built-in Providers

| Provider | Feature Flag | Status |
|----------|-------------|--------|
| Console | always on | Available |
| Resend | `provider-resend` | Phase 2 |
| SMTP | `provider-smtp` | Phase 2 |
| SendGrid | `provider-sendgrid` | Phase 3 |
| Postmark | `provider-postmark` | Phase 3 |
| AWS SES | `provider-ses` | Phase 3 |
| Mailgun | `provider-mailgun` | Phase 3 |

## License

MIT OR Apache-2.0
