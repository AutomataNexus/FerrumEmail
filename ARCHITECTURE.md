# Ferrum Email — Architecture

## Overview

Ferrum Email is a **component-based email framework for Rust** — the Rust equivalent of React Email + Resend. It provides type-safe, composable email templates with cross-client-compatible HTML rendering and a unified async sending API.

## Design Principles

1. **Type safety at compile time** — Email templates are Rust structs with typed props. No string interpolation, no unchecked template variables.
2. **Component composition** — Templates are built from composable components (like React components). A `Button` or `Footer` can be reused across any email.
3. **Email-client compatibility by default** — The renderer produces table-based layouts, inlines all CSS, and handles Outlook VML automatically. The developer never writes raw HTML tables.
4. **Zero-config sending** — One-liner sends via any supported provider. Feature flags keep compile times fast.
5. **Separation of concerns** — Templates know nothing about sending. Rendering knows nothing about providers. Each crate has a single responsibility.

## Crate Dependency Graph

```
                    Your Application
                          │
                    ┌─────▼──────┐
                    │ ferrum-     │
                    │ email-send  │   ← Sender, EmailProvider trait, providers
                    └─────┬──────┘
                          │ depends on
                    ┌─────▼──────┐
                    │ ferrum-     │
                    │ email-     │   ← Renderer, CSS inliner, text extractor
                    │ render     │
                    └─────┬──────┘
                          │ depends on
                    ┌─────▼──────┐
                    │ ferrum-     │
                    │ email-core │   ← Component trait, Node tree, Style, types
                    └────────────┘

    ferrum-email-components  ← Standard components (Html, Body, Button, etc.)
           │ depends on
    ferrum-email-core

    ferrum-email-preview     ← Dev preview server (Phase 2)
           │ depends on
    ferrum-email-core + ferrum-email-render

    ferrum-email-cli         ← CLI binary (Phase 2/3)
           │ depends on
    ferrum-email-core + ferrum-email-render + ferrum-email-send

    ferrum-email-macros      ← Proc macros: email!, register! (Phase 3)
```

## Core Abstractions

### The `Component` Trait (`ferrum-email-core`)

The fundamental building block. Every email template and reusable element implements this trait:

```rust
pub trait Component: Send + Sync {
    fn render(&self) -> Node;
    fn plain_text(&self) -> Option<String> { None }
    fn subject(&self) -> Option<&str> { None }
}
```

- `render()` produces a `Node` tree (the intermediate representation)
- `plain_text()` optionally overrides auto-generated plain text
- `subject()` provides the email subject line for top-level templates

### The `Node` Tree (`ferrum-email-core`)

An intermediate representation between components and rendered HTML:

```rust
pub enum Node {
    Element(Element),   // <tag attrs style>children</tag>
    Text(String),       // Raw text content
    Fragment(Vec<Node>),// Multiple nodes without a wrapper
    None,               // Empty/noop
}

pub struct Element {
    pub tag: Tag,           // Html, Body, Table, Td, P, A, etc.
    pub attrs: Vec<Attr>,   // (name, value) pairs
    pub style: Style,       // Typed inline CSS
    pub children: Vec<Node>,
}
```

This design decouples component logic from HTML emission, enabling:
- Different output formats (HTML, MJML, plain text)
- Style normalization and CSS inlining
- Client-specific transformations

### The `Style` System (`ferrum-email-core`)

All CSS properties are typed — no raw strings:

```rust
pub struct Style {
    pub font_family: Option<FontFamily>,
    pub font_size: Option<Px>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub padding: Option<Spacing>,
    // ... 20+ email-safe properties
}
```

The `Style::to_css()` method converts to an inline CSS string. Invalid CSS is impossible to construct.

### The `Renderer` (`ferrum-email-render`)

Walks a `Node` tree and emits email-safe HTML:

```
Component::render() → Node tree
       ↓
   CSS Inliner       (Style structs → style="" attributes)
       ↓
   HTML Emitter      (Node tree → HTML string with entity escaping)

   Text Extractor    (Node tree → plain text with link URLs, separators)
```

Key decisions:
- **No `<style>` blocks** — Gmail strips them. All CSS is inlined.
- **HTML entity escaping** — `&`, `<`, `>`, `"`, `'` in text and attributes
- **Void element handling** — `<img />`, `<hr />`, `<br />` are self-closing
- **DOCTYPE** — `<!DOCTYPE html>` is prepended by default

### The `Sender` (`ferrum-email-send`)

The user-facing API that ties rendering and sending together:

```rust
pub struct Sender {
    provider: Box<dyn EmailProvider>,
    default_from: Mailbox,
    renderer: Renderer,
}
```

One call does everything: render → build message → send via provider.

### The `EmailProvider` Trait (`ferrum-email-send`)

```rust
#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError>;
    async fn send_batch(&self, messages: Vec<EmailMessage>) -> Result<Vec<SendResult>, EmailError>;
}
```

Providers are pluggable. The `ConsoleProvider` prints to stdout for development.

## Component Library (`ferrum-email-components`)

All components use **table-based layouts** for Outlook compatibility:

| Component | HTML Output | Purpose |
|-----------|-------------|---------|
| `Html` | `<html lang dir xmlns>` | Root element |
| `Head` | `<head>` + meta tags | Document head |
| `Body` | `<body style>` | Document body |
| `Preview` | Hidden `<div>` with filler | Inbox preview text |
| `Container` | `<table align="center">` | Centered wrapper |
| `Section` | `<table width="100%"><tr><td>` | Full-width section |
| `Row` | `<table><tr>` | Multi-column row |
| `Column` | `<td>` | Table column |
| `Text` | `<p>` | Paragraph |
| `Heading` | `<h1>`–`<h6>` | Heading |
| `Button` | `<table><td><a>` | CTA button |
| `Link` | `<a>` | Hyperlink |
| `Image` | `<img>` | Image with dimensions |
| `Hr` | `<hr>` | Horizontal rule |
| `Code` | `<code>` | Inline code |
| `CodeBlock` | `<pre><code>` | Code block |
| `Spacer` | `<table><td height>` | Vertical space |

### Builder Pattern

Every component uses a fluent builder API:

```rust
Button::new("https://example.com", "Click Me")
    .background(Color::hex("C0392B"))
    .text_color(Color::white())
    .border_radius(Px(6))
    .into_node()
```

Components accept children via `.child(impl Component)` or `.child_node(Node)`.

## Email Client Compatibility Strategy

The framework targets these clients:

| Client | Rendering Engine | Strategy |
|--------|-----------------|----------|
| Gmail (web/mobile) | WebKit-based | All CSS inlined, no `<style>` blocks |
| Apple Mail | WebKit | Generally modern, best support |
| Outlook 2016–2021 | Word engine | Table-based layouts, VML for buttons |
| Outlook.com | WebKit | Mostly modern |
| Yahoo Mail | WebKit | Class stripping handled |
| Thunderbird | Gecko | Generally good |

The key guarantee: **developers write components, the renderer handles all compatibility.**

## Error Handling

Each layer has its own error type:

- `RenderError` — rendering failures (currently just `RenderFailed(String)`)
- `EmailError` — sending failures: `Render`, `Provider`, `InvalidAddress`, `MissingField`

`EmailError` implements `From<RenderError>` for seamless propagation.

## File Layout

```
ferrum-email/
├── Cargo.toml                       # Workspace manifest
├── PRD.md                           # Product requirements
├── ARCHITECTURE.md                  # This file
├── PROGRESS.md                      # Development progress tracker
├── README.md                        # Project README
│
├── crates/
│   ├── ferrum-email-core/           # Component trait, Node, Style, types
│   │   └── src/
│   │       ├── lib.rs               # Re-exports
│   │       ├── component.rs         # Component trait
│   │       ├── node.rs              # Node, Element, Tag, Attr
│   │       ├── style.rs             # Style struct, Border
│   │       ├── color.rs             # Color enum
│   │       ├── spacing.rs           # Spacing struct
│   │       └── types.rs             # Px, FontWeight, TextAlign, etc.
│   │
│   ├── ferrum-email-components/     # Standard component library
│   │   └── src/
│   │       ├── lib.rs               # Re-exports
│   │       ├── html.rs              # Html component
│   │       ├── head.rs              # Head component
│   │       ├── body.rs              # Body component
│   │       ├── preview.rs           # Preview text component
│   │       ├── container.rs         # Centered max-width container
│   │       ├── section.rs           # Full-width section
│   │       ├── row.rs               # Table row
│   │       ├── column.rs            # Table column
│   │       ├── text.rs              # Text/paragraph
│   │       ├── heading.rs           # H1-H6 headings
│   │       ├── button.rs            # CTA button (table-based)
│   │       ├── link.rs              # Anchor link
│   │       ├── image.rs             # Image
│   │       ├── hr.rs                # Horizontal rule
│   │       ├── code.rs              # Code + CodeBlock
│   │       └── spacer.rs            # Vertical spacer
│   │
│   ├── ferrum-email-render/         # Rendering engine
│   │   └── src/
│   │       ├── lib.rs               # Renderer, RenderError
│   │       ├── renderer.rs          # Core renderer
│   │       ├── html_emitter.rs      # HTML escaping utilities
│   │       ├── css_inliner.rs       # CSS inlining pass
│   │       └── text_extractor.rs    # Plain text extraction
│   │
│   ├── ferrum-email-send/           # Sending API
│   │   ├── src/
│   │   │   ├── lib.rs               # Re-exports
│   │   │   ├── error.rs             # EmailError
│   │   │   ├── message.rs           # EmailMessage, Mailbox, Attachment
│   │   │   ├── provider.rs          # EmailProvider trait
│   │   │   ├── sender.rs            # Sender struct
│   │   │   └── providers/
│   │   │       ├── mod.rs
│   │   │       └── console.rs       # ConsoleProvider
│   │   └── tests/
│   │       └── integration.rs       # 23 integration tests
│   │
│   ├── ferrum-email-preview/        # Live preview server (Phase 2)
│   ├── ferrum-email-macros/         # Proc macros (Phase 3)
│   └── ferrum-email-cli/            # CLI binary (Phase 2/3)
│
└── examples/
    ├── welcome/                     # Welcome email with footer composition
    └── password-reset/              # Password reset with security info
```

## Testing Strategy

- **Unit tests** in each crate's source files (renderer, text extractor, HTML emitter, mailbox parsing)
- **Integration tests** in `ferrum-email-send/tests/integration.rs` — full pipeline tests
- **Doc tests** — runnable examples in documentation
- **Total: 39 tests, all passing**

## Future Architecture (Phases 2–4)

### Phase 2: Providers + Preview
- Resend, SMTP (via lettre), Mailtrap providers
- Live preview server (axum + notify for hot-reload)
- Outlook VML generation

### Phase 3: Full Provider Suite + Macros
- SendGrid, Postmark, AWS SES, Mailgun providers
- `email!` proc macro for JSX-like syntax
- CLI: validate, send, new subcommands
- Client compatibility matrix with warnings

### Phase 4: Open Source Release
- Full docs.rs documentation
- Crates.io publishing
- CI/CD pipeline
