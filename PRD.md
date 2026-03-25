# Ferrum Email — Component-Based Email Framework for Rust
## Product Requirements Document v1.0
**Owner:** Andrew Jewell Sr. — AutomataNexus LLC  
**Contact:** devops@automatanexus.com  
**Classification:** Internal / Open Source Candidate  
**Status:** Pre-Development  
**Last Updated:** 2026-03-24

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Background & Motivation](#2-background--motivation)
3. [The Gap This Fills](#3-the-gap-this-fills)
4. [Architecture Overview](#4-architecture-overview)
5. [Core Crates](#5-core-crates)
   - 5.1 [ferrum-email-core](#51-ferrum-email-core)
   - 5.2 [ferrum-email-components](#52-ferrum-email-components)
   - 5.3 [ferrum-email-render](#53-ferrum-email-render)
   - 5.4 [ferrum-email-send](#54-ferrum-email-send)
   - 5.5 [ferrum-email-preview](#55-ferrum-email-preview)
   - 5.6 [ferrum-email-cli](#56-ferrum-email-cli)
6. [Component System Design](#6-component-system-design)
7. [Provider Abstraction](#7-provider-abstraction)
8. [Template API Design](#8-template-api-design)
9. [Developer Experience](#9-developer-experience)
10. [Email Client Compatibility](#10-email-client-compatibility)
11. [Tech Stack](#11-tech-stack)
12. [Directory Structure](#12-directory-structure)
13. [Development Phases](#13-development-phases)
14. [Test Strategy](#14-test-strategy)
15. [Acceptance Criteria](#15-acceptance-criteria)
16. [Open Source Strategy](#16-open-source-strategy)

---

## 1. Executive Summary

Ferrum Email is a **component-based email templating and sending framework for Rust** — the Rust equivalent of React Email + Resend. It lets developers define type-safe, composable email templates using a Rust component system, render them to cross-client-compatible HTML, and send them through any email provider (Resend, SendGrid, Postmark, AWS SES, SMTP) via a unified async API.

The entire experience is designed around the Rust developer: templates are pure Rust structs, props are typed at compile time, rendering is zero-allocation where possible, and the live preview server hot-reloads as you save your template files. No string concatenation, no unchecked `{{ variable }}` handlebars, no raw HTML table hell.

This is the library that does not exist yet. The Rust email ecosystem has `lettre` (SMTP sending), `mrml` (MJML rendering), and general-purpose templating engines — but nothing that ties them together into a first-class developer experience. Ferrum Email is that library.

---

## 2. Background & Motivation

Every serious Rust backend application eventually needs to send emails: welcome messages, password resets, notifications, invoices, alerts. The current state of Rust email in 2026:

- **`lettre`** — excellent SMTP mailer, no templating
- **`mrml`** — fast MJML renderer, but you write raw MJML markup by hand
- **`handlebars` / `tera` / `minijinja`** — general templating engines bolted onto HTML strings, no type safety, no component abstraction, no email-client awareness
- **`mail-template`** — tiny niche crate, not production-grade

The Node.js ecosystem has React Email: type-safe JSX components, live preview with hot reload, cross-client compatibility handled automatically, and one-liner sends via Resend. The Python ecosystem has at least a handful of decent templating approaches with Jinja2. Rust has nothing coherent.

Ferrum Email fixes this. It is built for the AutomataNexus stack first — NexusBMS transactional emails, Aegis-DB alert notifications, NexusProbe security finding digests — and designed to be open-sourced as a general-purpose library that the Rust community has been missing.

---

## 3. The Gap This Fills

| Capability | React Email (JS) | lettre (Rust) | mrml (Rust) | Ferrum Email (Rust) |
|-----------|-----------------|---------------|-------------|---------------------|
| Component-based templates | ✅ | ❌ | ❌ | ✅ |
| Type-safe props | ✅ (TypeScript) | ❌ | ❌ | ✅ (Rust types) |
| Cross-client compatibility | ✅ | ❌ | ✅ (via MJML) | ✅ |
| Live preview server | ✅ | ❌ | ❌ | ✅ |
| Plain text auto-generation | ✅ | Manual | ❌ | ✅ |
| Multi-provider sending API | Via Resend SDK | SMTP only | ❌ | ✅ |
| Async sending | ✅ | ✅ | N/A | ✅ |
| Zero external runtime deps | ❌ (Node) | ✅ | ✅ | ✅ |
| Compile-time template validation | ❌ | ❌ | ❌ | ✅ |

---

## 4. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Rust Application                     │
│                                                             │
│   let email = WelcomeEmail { name: "Andrew".into(),         │
│                              verify_url: url };             │
│   sender.send(email, "andrew@example.com").await?;          │
└──────────────────────────┬──────────────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │    ferrum-email-send     │
              │   Provider abstraction   │
              │  Resend | SES | SMTP...  │
              └────────────┬────────────┘
                           │
              ┌────────────▼────────────┐
              │   ferrum-email-render    │
              │  Component tree → HTML   │
              │  + plain text fallback   │
              │  + inlined CSS           │
              └────────────┬────────────┘
                           │
        ┌──────────────────▼──────────────────┐
        │        ferrum-email-components        │
        │  Html, Head, Body, Section, Row,      │
        │  Column, Text, Button, Image, Hr,     │
        │  Link, Code, Preview, Container...    │
        └──────────────────┬──────────────────┘
                           │
              ┌────────────▼────────────┐
              │    ferrum-email-core     │
              │  Component trait,        │
              │  Props, Node tree,       │
              │  Style system            │
              └─────────────────────────┘
```

---

## 5. Core Crates

### 5.1 ferrum-email-core

The foundation. Defines the `Component` trait, the `Node` tree (analogous to a virtual DOM for email), the `Props` system, and the `Style` type.

**The `Component` Trait:**

```rust
pub trait Component: Send + Sync {
    /// Render this component into a Node tree.
    fn render(&self) -> Node;

    /// Optional: override to provide a custom plain text version.
    /// Default implementation extracts text from the Node tree.
    fn plain_text(&self) -> Option<String> {
        None
    }

    /// The email subject line. Only meaningful on top-level email components.
    fn subject(&self) -> Option<&str> {
        None
    }
}
```

**The `Node` Tree:**

```rust
pub enum Node {
    Element(Element),
    Text(String),
    Fragment(Vec<Node>),
    None,
}

pub struct Element {
    pub tag: Tag,               // Html, Body, Table, Td, etc.
    pub attrs: Vec<Attr>,       // (name, value) pairs
    pub style: Style,           // inline style map
    pub children: Vec<Node>,
}
```

The Node tree is an intermediate representation. Components compose Node trees. The renderer walks the tree and emits email-safe HTML.

**The `Style` System:**

Styles are typed, not string maps:

```rust
pub struct Style {
    pub font_family:    Option<FontFamily>,
    pub font_size:      Option<Px>,
    pub font_weight:    Option<FontWeight>,
    pub color:          Option<Color>,
    pub background:     Option<Color>,
    pub padding:        Option<Spacing>,
    pub margin:         Option<Spacing>,
    pub border_radius:  Option<Px>,
    pub width:          Option<SizeValue>,
    pub max_width:      Option<SizeValue>,
    pub text_align:     Option<TextAlign>,
    pub line_height:    Option<LineHeight>,
    pub display:        Option<Display>,
    // ... full set of email-safe CSS properties
}
```

All style values are type-checked at compile time. Invalid combinations (e.g., properties that break in Outlook) can produce compile-time warnings via a proc macro.

---

### 5.2 ferrum-email-components

The standard component library. Every component is a Rust struct implementing `Component`. This is the equivalent of `@react-email/components`.

**Built-in Components:**

| Component | Purpose |
|-----------|---------|
| `Html` | Root element with lang and dir |
| `Head` | Head section with meta charset, viewport |
| `Preview` | Preview text shown in email client inbox list |
| `Body` | Body with background color |
| `Container` | Centered max-width wrapper (table-based) |
| `Section` | Full-width row section |
| `Row` | Table row |
| `Column` | Table column with configurable width |
| `Text` | Paragraph with typography props |
| `Heading` | H1–H6 with size presets |
| `Button` | CTA button (table-based for Outlook compat) |
| `Link` | Anchor tag with tracking-safe href |
| `Image` | Img with alt, width, height (required for clients) |
| `Hr` | Horizontal rule divider |
| `Code` | Inline code span |
| `CodeBlock` | Monospace code block |
| `Spacer` | Vertical whitespace |
| `Tailwind` | Wrapper enabling Tailwind-style utility classes |

Each component exposes typed props as struct fields:

```rust
pub struct Button {
    pub href:             String,
    pub children:         String,
    pub background_color: Color,
    pub text_color:       Color,
    pub border_radius:    Px,
    pub padding:          Spacing,
    pub font_size:        Px,
    pub font_weight:      FontWeight,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            href:             String::new(),
            children:         String::new(),
            background_color: Color::hex("000000"),
            text_color:       Color::hex("ffffff"),
            border_radius:    Px(4),
            padding:          Spacing::xy(Px(12), Px(20)),
            font_size:        Px(14),
            font_weight:      FontWeight::SemiBold,
        }
    }
}

impl Component for Button {
    fn render(&self) -> Node {
        // Renders as a table-based button for Outlook compatibility
        // See rendering section for the table-in-VML approach
    }
}
```

---

### 5.3 ferrum-email-render

Walks a `Node` tree produced by a `Component` and emits:

1. **HTML** — email-safe, cross-client-compatible HTML with inlined CSS
2. **Plain text** — extracted from text nodes with sensible formatting
3. **MJML** (optional feature flag) — intermediate MJML for use with `mrml` if preferred

**Rendering pipeline:**

```
Component::render() → Node tree
       ↓
   Normalizer        (collapse fragments, resolve defaults)
       ↓
   CSS Inliner       (move all styles to inline style="" attrs)
       ↓
   Table Builder     (flex/grid → table layouts for Outlook)
       ↓
   VML Generator     (conditional comments for Outlook buttons/images)
       ↓
   HTML Emitter      (produces final HTML string)
       ↓
   Text Extractor    (produces plain text fallback)
```

**CSS Inlining:**

All styles are inlined automatically — no `<style>` blocks in the `<head>` (Gmail strips them). The inliner handles specificity, merges inherited styles, and only emits properties that the target email client supports.

**Outlook VML:**

Buttons and background images require VML conditional comments for Outlook. The renderer handles this automatically — the developer never writes VML. A `Button` component just renders correctly in Outlook without the developer knowing VML exists.

**Client Compatibility Matrix:**

The renderer maintains a compatibility matrix (configurable target clients) and will:
- Warn at render time if a style property is unsupported by a targeted client
- Automatically generate fallbacks where possible
- Strip unsupported properties cleanly where fallbacks aren't possible

---

### 5.4 ferrum-email-send

The provider abstraction layer. Defines the `EmailProvider` trait and implements it for every major provider.

**The `EmailProvider` Trait:**

```rust
#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError>;
    async fn send_batch(&self, messages: Vec<EmailMessage>) -> Result<Vec<SendResult>, EmailError>;
}

pub struct EmailMessage {
    pub from:        Mailbox,           // "Name <email@domain.com>"
    pub to:          Vec<Mailbox>,
    pub cc:          Vec<Mailbox>,
    pub bcc:         Vec<Mailbox>,
    pub reply_to:    Option<Mailbox>,
    pub subject:     String,
    pub html:        String,
    pub text:        Option<String>,
    pub attachments: Vec<Attachment>,
    pub headers:     Vec<(String, String)>,
    pub tags:        Vec<Tag>,          // provider-specific tagging
}

pub struct SendResult {
    pub message_id: String,
    pub provider:   String,
}
```

**Built-in Providers:**

| Provider | Feature Flag | Notes |
|----------|-------------|-------|
| Resend | `provider-resend` | REST API, webhooks, tagging |
| SendGrid | `provider-sendgrid` | REST API |
| Postmark | `provider-postmark` | REST API, template support |
| AWS SES | `provider-ses` | AWS SDK integration |
| Mailgun | `provider-mailgun` | REST API |
| SMTP | `provider-smtp` | Via lettre, any SMTP server |
| Mailtrap | `provider-mailtrap` | Dev/staging testing |
| Console | always on | Prints to stdout, for local dev |

Each provider is behind a feature flag so you only compile what you need.

**The `Sender` struct:**

The main user-facing API:

```rust
pub struct Sender {
    provider: Box<dyn EmailProvider>,
    default_from: Mailbox,
}

impl Sender {
    pub fn new(provider: impl EmailProvider + 'static, from: impl Into<Mailbox>) -> Self

    /// Send any type that implements Component
    pub async fn send<C: Component>(
        &self,
        component: C,
        to: impl Into<Mailbox>,
    ) -> Result<SendResult, EmailError>

    /// Send with full control over all message fields
    pub async fn send_message(
        &self,
        message: EmailMessage,
    ) -> Result<SendResult, EmailError>

    /// Send to multiple recipients
    pub async fn send_batch<C: Component>(
        &self,
        component: C,
        recipients: Vec<Mailbox>,
    ) -> Result<Vec<SendResult>, EmailError>
}
```

**Usage example:**

```rust
use ferrum_email_send::{Sender, providers::ResendProvider};

let provider = ResendProvider::new(std::env::var("RESEND_API_KEY")?);
let sender = Sender::new(provider, "AutomataNexus <no-reply@automatanexus.com>");

sender.send(
    WelcomeEmail { name: "Andrew".into(), verify_url: url },
    "andrew@example.com",
).await?;
```

---

### 5.5 ferrum-email-preview

A local development preview server — the equivalent of `email dev` from React Email. Run it, open your browser, see your email rendered live with hot-reload as you save your template files.

**Capabilities:**

- Spawns an HTTP server (default port 3737)
- Scans a configured directory for types implementing `Component` via a registration macro
- Renders each registered template to HTML and serves it in a browser preview UI
- Watches source files and re-renders on save (via `notify` crate)
- Injects a small hot-reload snippet into the preview HTML (WebSocket-based, not in production render)
- Preview UI shows:
  - Desktop viewport (600px email width)
  - Mobile viewport (375px)
  - Plain text version
  - Raw HTML source view
  - Dark mode toggle
  - Email client simulation selector (Gmail, Outlook, Apple Mail, etc. — CSS adjustments to simulate client quirks)
- Props editor: auto-generates a UI form for the template's props using their types, allowing you to change prop values in the browser and see the template update live

**Registration macro:**

```rust
// In your templates module
ferrum_preview_register!(
    WelcomeEmail,
    PasswordResetEmail,
    SecurityAlertEmail,
    InvoiceEmail,
);
```

**CLI invocation:**

```
ferrum preview --dir ./src/emails --port 3737
```

---

### 5.6 ferrum-email-cli

The `ferrum` CLI binary. Installed via `cargo install ferrum-email-cli`.

```
ferrum <SUBCOMMAND>

SUBCOMMANDS:
    preview     Start the live preview server
    render      Render a template to HTML/text/mjml and write to file
    send        Send a template via a configured provider (for testing)
    validate    Validate templates against email client compatibility matrix
    new         Scaffold a new email template file

EXAMPLES:
    ferrum preview --dir ./src/emails
    ferrum render --template WelcomeEmail --format html --output ./out/welcome.html
    ferrum validate --dir ./src/emails --clients gmail,outlook,apple-mail
    ferrum new WelcomeEmail --dir ./src/emails
```

---

## 6. Component System Design

### 6.1 Defining a Template

A complete email template is a Rust struct implementing `Component`:

```rust
use ferrum_email_components::*;
use ferrum_email_core::Component;

pub struct WelcomeEmail {
    pub name:        String,
    pub verify_url:  String,
    pub app_name:    String,
}

impl Component for WelcomeEmail {
    fn subject(&self) -> Option<&str> {
        Some("Welcome to AutomataNexus")
    }

    fn render(&self) -> Node {
        Html::new()
            .child(Head::new())
            .child(Preview::new(format!("Welcome, {}! Verify your email to get started.", self.name)))
            .child(
                Body::new().background(Color::hex("f6f6f6")).child(
                    Container::new().max_width(Px(600)).child(
                        Section::new().padding(Spacing::all(Px(32))).children([
                            Heading::h1(&format!("Welcome, {}!", self.name))
                                .color(Color::hex("1a1a1a")),
                            Text::new(&format!(
                                "Thanks for signing up for {}. Click the button below to verify your email address.",
                                self.app_name
                            ))
                            .color(Color::hex("555555"))
                            .font_size(Px(16))
                            .line_height(1.6),
                            Spacer::new(Px(24)),
                            Button::new(&self.verify_url, "Verify Email Address")
                                .background(Color::hex("C0392B"))
                                .text_color(Color::white())
                                .border_radius(Px(6)),
                            Spacer::new(Px(24)),
                            Text::new("If you didn't sign up for this account, you can safely ignore this email.")
                                .color(Color::hex("999999"))
                                .font_size(Px(13)),
                        ])
                    )
                )
            )
            .into_node()
    }
}
```

### 6.2 Composition

Components compose naturally — a component's `render()` method can instantiate and render other components:

```rust
pub struct EmailFooter {
    pub company: String,
    pub address: String,
    pub unsubscribe_url: String,
}

impl Component for EmailFooter {
    fn render(&self) -> Node {
        Section::new()
            .background(Color::hex("f0f0f0"))
            .padding(Spacing::all(Px(20)))
            .children([
                Hr::new().color(Color::hex("dddddd")),
                Text::new(&self.company).font_size(Px(12)).color(Color::hex("aaaaaa")),
                Text::new(&self.address).font_size(Px(12)).color(Color::hex("aaaaaa")),
                Link::new(&self.unsubscribe_url, "Unsubscribe")
                    .font_size(Px(12))
                    .color(Color::hex("aaaaaa")),
            ])
            .into_node()
    }
}

// Used in another template:
impl Component for WelcomeEmail {
    fn render(&self) -> Node {
        Html::new().child(
            Body::new().children([
                // ... main content ...
                EmailFooter {
                    company: "AutomataNexus LLC".into(),
                    address: "Warsaw, IN".into(),
                    unsubscribe_url: self.unsubscribe_url.clone(),
                }.render(),  // compose directly
            ])
        ).into_node()
    }
}
```

### 6.3 The `email!` Macro (Optional Ergonomic Sugar)

For developers who want a more declarative feel, an optional proc macro:

```rust
let node = email! {
    <Container max_width=600>
        <Heading level=1 color="#1a1a1a">{ &self.title }</Heading>
        <Text font_size=16>{ &self.body }</Text>
        <Button href={ &self.cta_url } background="#C0392B">
            { &self.cta_label }
        </Button>
    </Container>
};
```

This compiles to the same builder chain as the direct API — it's purely ergonomic sugar. The direct builder API is always the canonical form.

---

## 7. Provider Abstraction

### 7.1 Resend Provider

```rust
use ferrum_email_send::providers::ResendProvider;

let provider = ResendProvider::builder()
    .api_key(std::env::var("RESEND_API_KEY")?)
    .region(ResendRegion::UsEast1)  // optional, for latency routing
    .build();
```

Supports:
- Single and batch sends
- Tags for email categorization
- Idempotency keys
- Webhook event types (open, click, bounce, complaint) — Ferrum provides typed webhook payload structs for Axum/Actix handlers

### 7.2 SMTP Provider (via lettre)

```rust
use ferrum_email_send::providers::SmtpProvider;

let provider = SmtpProvider::builder()
    .host("smtp.example.com")
    .port(587)
    .credentials("user@example.com", std::env::var("SMTP_PASSWORD")?)
    .starttls(true)
    .build()
    .await?;
```

### 7.3 Console Provider (always available, no feature flag)

Prints the full rendered email to stdout with ANSI formatting. Used in development and tests.

```rust
use ferrum_email_send::providers::ConsoleProvider;

let provider = ConsoleProvider::new();
// Sending prints to stdout instead of making any network call.
```

### 7.4 Adding a Custom Provider

Implement the trait:

```rust
struct MyCustomProvider { /* ... */ }

#[async_trait]
impl EmailProvider for MyCustomProvider {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        // your implementation
    }

    async fn send_batch(&self, messages: Vec<EmailMessage>) -> Result<Vec<SendResult>, EmailError> {
        // your implementation
    }
}
```

---

## 8. Template API Design

### 8.1 Subject Lines

Subject is a method on `Component`, not a separate concept:

```rust
impl Component for PasswordResetEmail {
    fn subject(&self) -> Option<&str> {
        Some("Reset your password")
    }
    // ...
}
```

The `Sender` automatically uses this as the email subject. For dynamic subjects:

```rust
fn subject(&self) -> Option<&str> {
    // Can't return a reference to a temporary — use a field:
    Some(&self.subject_line)
}
```

### 8.2 Attachments

Attachments are added at send time, not in the template:

```rust
sender.send_message(EmailMessage {
    from: "no-reply@automatanexus.com".parse()?,
    to: vec!["andrew@example.com".parse()?],
    subject: "Your security report".into(),
    html: renderer.render_html(&SecurityReportEmail { /* ... */ })?,
    text: renderer.render_text(&SecurityReportEmail { /* ... */ }),
    attachments: vec![
        Attachment::from_file("report.pdf", "./reports/2026-03.pdf")?,
        Attachment::from_bytes("data.json", json_bytes, "application/json"),
    ],
    ..Default::default()
}).await?;
```

### 8.3 Internationalization

Templates support locale-aware rendering via an optional `Locale` parameter:

```rust
pub struct WelcomeEmail {
    pub name: String,
    pub locale: Locale,    // Locale::EnUs, Locale::Es, etc.
}
```

The `Html` component sets `lang` and `dir` based on the locale automatically. String content is the template author's responsibility — Ferrum provides the locale propagation mechanism, not translation strings.

---

## 9. Developer Experience

### 9.1 Getting Started

```toml
[dependencies]
ferrum-email-core       = "0.1"
ferrum-email-components = "0.1"
ferrum-email-send       = { version = "0.1", features = ["provider-resend"] }

[dev-dependencies]
ferrum-email-preview    = "0.1"
```

### 9.2 Zero Boilerplate First Email

```rust
use ferrum_email_components::*;
use ferrum_email_core::Component;
use ferrum_email_send::{Sender, providers::ConsoleProvider};

struct HelloEmail { pub name: String }

impl Component for HelloEmail {
    fn subject(&self) -> Option<&str> { Some("Hello!") }
    fn render(&self) -> Node {
        Html::default()
            .child(Body::default()
                .child(Text::new(&format!("Hello, {}!", self.name))))
            .into_node()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sender = Sender::new(ConsoleProvider::new(), "me@example.com");
    sender.send(HelloEmail { name: "Andrew".into() }, "andrew@example.com").await?;
    Ok(())
}
```

That's it. No boilerplate. No configuration files. No HTML strings.

### 9.3 Error Messages

Compiler errors from mis-typed props should be clear and point directly to the field:

```
error[E0308]: mismatched types
  --> src/emails/welcome.rs:14:32
   |
14 |             Button::new(&self.url, 42)
   |                                   ^^ expected `&str`, found integer
```

This is just Rust's normal type system — the framework gets this for free.

### 9.4 Testing

Templates are pure Rust structs — they're trivially testable:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_email_render::Renderer;

    #[test]
    fn welcome_email_renders_without_panic() {
        let email = WelcomeEmail {
            name: "Test User".into(),
            verify_url: "https://example.com/verify/abc123".into(),
            app_name: "TestApp".into(),
        };
        let renderer = Renderer::default();
        let html = renderer.render_html(&email).unwrap();
        assert!(html.contains("Test User"));
        assert!(html.contains("https://example.com/verify/abc123"));
    }

    #[test]
    fn welcome_email_subject() {
        let email = WelcomeEmail { /* ... */ };
        assert_eq!(email.subject(), Some("Welcome to AutomataNexus"));
    }

    #[test]
    fn welcome_email_plain_text_has_no_html_tags() {
        let email = WelcomeEmail { /* ... */ };
        let renderer = Renderer::default();
        let text = renderer.render_text(&email).unwrap();
        assert!(!text.contains('<'));
        assert!(!text.contains('>'));
    }
}
```

---

## 10. Email Client Compatibility

Ferrum's renderer targets the following clients by default. The compatibility matrix is configurable — if you only care about modern clients you can loosen constraints:

| Client | Rendering Engine | Known Issues Handled |
|--------|-----------------|---------------------|
| Gmail (web) | WebKit-based | No `<style>` in `<head>`, no `<link>`, strips class attributes |
| Gmail (mobile) | Same | Same |
| Apple Mail | WebKit | Generally modern, best support |
| Outlook 2016–2021 | Word rendering engine | No CSS, requires table layouts and VML for buttons/images |
| Outlook.com | WebKit | Mostly modern, some quirks |
| Yahoo Mail | WebKit | Some class stripping |
| Samsung Mail | WebKit | Limited CSS |
| Thunderbird | Gecko | Generally good |

**The developer writes components. The renderer deals with all of this.** That's the value proposition.

---

## 11. Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Language | Rust 2024 edition | The whole point |
| Async runtime | tokio | Standard async Rust |
| HTTP (sending) | reqwest | Async HTTP for REST provider APIs |
| HTTP (preview server) | axum | Lightweight, tokio-native |
| File watching (preview) | notify | Cross-platform file system events |
| HTML parsing (render) | Custom tree walker | Control over output, no external parser needed |
| CSS inlining | Custom | Email-specific subset, don't need a full CSS parser |
| Proc macros | syn + quote | `email!` macro and registration macros |
| Serialization | serde + serde_json | Provider API payloads |
| SMTP | lettre | Proven, well-maintained |
| Testing | Rust built-in + insta | Snapshot testing for rendered HTML |
| CI | GitHub Actions | cargo test, clippy, audit |

---

## 12. Directory Structure

```
ferrum-email/
├── Cargo.toml                       # Workspace manifest
├── Cargo.lock
├── README.md
├── PRD.md                           # This file
├── CHANGELOG.md
│
├── crates/
│   ├── ferrum-email-core/           # Component trait, Node tree, Style system
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── component.rs
│   │       ├── node.rs
│   │       ├── style.rs
│   │       ├── color.rs
│   │       ├── spacing.rs
│   │       └── types.rs             # Px, FontWeight, TextAlign, etc.
│   │
│   ├── ferrum-email-components/     # Standard component library
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── html.rs
│   │       ├── head.rs
│   │       ├── body.rs
│   │       ├── container.rs
│   │       ├── section.rs
│   │       ├── row.rs
│   │       ├── column.rs
│   │       ├── text.rs
│   │       ├── heading.rs
│   │       ├── button.rs
│   │       ├── link.rs
│   │       ├── image.rs
│   │       ├── hr.rs
│   │       ├── code.rs
│   │       ├── spacer.rs
│   │       ├── preview.rs
│   │       └── tailwind.rs
│   │
│   ├── ferrum-email-render/         # Node tree → HTML + plain text
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs
│   │       ├── css_inliner.rs
│   │       ├── table_builder.rs     # flex/grid → table layout conversion
│   │       ├── vml.rs               # Outlook VML generation
│   │       ├── text_extractor.rs    # plain text fallback
│   │       └── compat/
│   │           ├── mod.rs
│   │           ├── gmail.rs
│   │           ├── outlook.rs
│   │           └── matrix.rs        # compatibility matrix
│   │
│   ├── ferrum-email-send/           # Provider abstraction + all providers
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs          # EmailProvider trait
│   │       ├── message.rs           # EmailMessage, Mailbox, Attachment
│   │       ├── sender.rs            # Sender struct
│   │       └── providers/
│   │           ├── console.rs
│   │           ├── smtp.rs
│   │           ├── resend.rs
│   │           ├── sendgrid.rs
│   │           ├── postmark.rs
│   │           ├── ses.rs
│   │           ├── mailgun.rs
│   │           └── mailtrap.rs
│   │
│   ├── ferrum-email-preview/        # Live preview server
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs            # axum server
│   │       ├── watcher.rs           # file system watcher
│   │       ├── registry.rs          # template registration
│   │       └── ui/                  # preview UI HTML/CSS/JS (embedded)
│   │
│   ├── ferrum-email-macros/         # Proc macros: email!, ferrum_preview_register!
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── ferrum-email-cli/            # ferrum CLI binary
│       └── src/
│           └── main.rs
│
├── examples/
│   ├── welcome/                     # Complete welcome email example
│   ├── password-reset/
│   ├── invoice/
│   ├── security-alert/              # NexusProbe-style security digest email
│   └── newsletter/
│
└── tests/
    ├── snapshots/                   # insta snapshot files
    └── integration/                 # send tests against Mailtrap
```

---

## 13. Development Phases

### Phase 1 — Core + Components + Render (Weeks 1–5)

- `ferrum-email-core`: Component trait, Node tree, Style system, all primitive types
- `ferrum-email-components`: all standard components (Html through Tailwind)
- `ferrum-email-render`: HTML emitter, CSS inliner, plain text extractor
- `ferrum-email-send`: EmailProvider trait, EmailMessage, Sender, ConsoleProvider
- Basic snapshot tests for every component

End state: A complete email template can be written, rendered to HTML, and printed to console. No preview server, no external providers yet.

### Phase 2 — Providers + Preview (Weeks 6–9)

- `ferrum-email-send`: Resend provider, SMTP provider (lettre), Mailtrap provider
- `ferrum-email-preview`: live preview server with hot reload, desktop/mobile viewports, plain text view, raw HTML view
- `ferrum-email-macros`: `ferrum_preview_register!` macro
- `ferrum-email-cli`: `ferrum preview` and `ferrum render` subcommands
- Outlook VML generation in renderer (buttons and background images)

End state: `ferrum preview --dir ./src/emails` opens a browser, shows all registered templates, hot-reloads on save. Templates can be sent via Resend or SMTP.

### Phase 3 — Full Provider Suite + Macros + Polish (Weeks 10–13)

- Remaining providers: SendGrid, Postmark, AWS SES, Mailgun
- `ferrum-email-macros`: `email!` proc macro
- `ferrum-email-cli`: `ferrum validate`, `ferrum send`, `ferrum new`
- Preview server props editor UI
- Email client simulation in preview (Gmail, Outlook, Apple Mail modes)
- Full compatibility matrix with warnings at render time
- Complete example suite

End state: Full feature parity with React Email + Resend for the Rust ecosystem.

### Phase 4 — Open Source Release Prep (Weeks 14–16)

- Documentation: docs.rs-compatible doc comments on every public API
- README with quick-start, comparison to React Email, migration guide
- CHANGELOG
- Crates.io publish for all crates
- GitHub repo under AutomataNexus organization
- CI/CD: cargo test, clippy, audit, doc build, crates.io publish on tag

---

## 14. Test Strategy

### Unit Tests

- Every component renders to valid HTML without panicking
- Style inlining correctly moves all styles to inline attributes
- Plain text extraction removes all HTML tags
- VML generation produces valid VML for Outlook buttons
- Provider implementations correctly serialize EmailMessage to their API format

### Snapshot Tests (insta)

Every component has a snapshot test for its rendered HTML output. Snapshot changes require explicit review and approval — this prevents silent rendering regressions.

```rust
#[test]
fn button_renders_correctly() {
    let button = Button {
        href: "https://example.com".into(),
        children: "Click Me".into(),
        ..Default::default()
    };
    let renderer = Renderer::default();
    let html = renderer.render_html(&button).unwrap();
    insta::assert_snapshot!(html);
}
```

### Integration Tests

- Send a test email via Mailtrap provider and verify it arrives
- Preview server serves correct HTML for registered templates
- Hot-reload correctly updates rendered output after template file change

### CI Pipeline

```yaml
jobs:
  test:
    - cargo test --workspace --all-features
    - cargo clippy --workspace --all-features -- -D warnings
    - cargo audit
    - cargo doc --workspace --no-deps
  integration:
    - cargo test --test integration  # requires MAILTRAP_API_KEY env var
```

---

## 15. Acceptance Criteria

### Phase 1 Complete When:

- [ ] A complete email template (Html → Body → Container → Section → Text + Button) renders to valid HTML
- [ ] CSS inliner moves all styles to inline `style=""` attributes — no `<style>` blocks in output
- [ ] Plain text extractor produces clean text with no HTML tags from any rendered template
- [ ] All component snapshot tests pass
- [ ] ConsoleProvider prints a legible rendered email to stdout

### Phase 2 Complete When:

- [ ] `ferrum preview` serves all registered templates in a browser with hot-reload
- [ ] Resend provider successfully sends an email to a real inbox in integration test
- [ ] SMTP provider sends via Mailtrap in integration test
- [ ] Button component renders correctly in Outlook (VML verified via snapshot)

### Phase 3 Complete When:

- [ ] All six providers pass integration tests
- [ ] `email!` macro compiles and produces identical output to the builder API
- [ ] `ferrum validate` correctly identifies an unsupported CSS property for a targeted client
- [ ] Preview server props editor changes props and re-renders the template live
- [ ] All snapshot tests pass with full --all-features build

### Phase 4 Complete When:

- [ ] All public APIs have doc comments, `cargo doc` builds without warnings
- [ ] All six crates publish successfully to crates.io
- [ ] README quick-start gets a complete email sent in under 10 lines of Rust
- [ ] CI passes on every commit to main

---

## 16. Open Source Strategy

**Repository:** `github.com/AutomataNexus/ferrum-email`

**License:** MIT OR Apache-2.0 (standard Rust dual license)

**Crates.io names:** `ferrum-email-core`, `ferrum-email-components`, `ferrum-email-render`, `ferrum-email-send`, `ferrum-email-preview`, `ferrum-email-cli`

**Launch strategy:**
- Post on Hacker News as "Show HN: Ferrum Email — React Email for Rust"
- Post on r/rust
- DEV.to article: "Building type-safe email templates in Rust"
- Reach out to lettre maintainers for potential collaboration or endorsement

**Why this wins:**
- The Rust community has been asking for this. The need shows up repeatedly on r/rust, in GitHub issues on lettre, and in Discord servers. Nobody has built it properly.
- The component-based approach with type-safe props is a genuinely better DX than anything currently available in any language except React Email.
- AutomataNexus gets a real library for its own use that also becomes a community contribution.

---

*Ferrum Email — Andrew Jewell Sr. — AutomataNexus LLC — devops@automatanexus.com*