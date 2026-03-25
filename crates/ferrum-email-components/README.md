# ferrum-email-components

The standard component library for [Ferrum Email](https://github.com/AutomataNexus/ferrum-email).

## Components

| Component | Purpose | HTML Output |
|-----------|---------|-------------|
| `Html` | Root element | `<html lang dir xmlns>` |
| `Head` | Document head | `<head>` + meta tags |
| `Body` | Document body | `<body style>` |
| `Preview` | Inbox preview text | Hidden `<div>` with filler |
| `Container` | Centered wrapper | `<table align="center">` |
| `Section` | Full-width section | `<table width="100%"><tr><td>` |
| `Row` | Multi-column row | `<table><tr>` |
| `Column` | Table column | `<td width>` |
| `Text` | Paragraph | `<p style>` |
| `Heading` | H1-H6 | `<h1>`–`<h6>` |
| `Button` | CTA button | `<table><td><a>` |
| `Link` | Hyperlink | `<a href>` |
| `Image` | Image | `<img src alt width>` |
| `Hr` | Divider | `<hr>` |
| `Code` | Inline code | `<code>` |
| `CodeBlock` | Code block | `<pre><code>` |
| `Spacer` | Vertical space | `<table><td height>` |

## Usage

All components use a builder pattern:

```rust
use ferrum_email_components::*;
use ferrum_email_core::Component;

let email = Html::new()
    .child(Head::new().title("My Email"))
    .child(Body::new().background(Color::hex("f6f6f6")).child(
        Container::new().max_width(Px(600)).child(
            Section::new().padding(Spacing::all(Px(32)))
                .child_node(Heading::h1("Hello!").color(Color::hex("1a1a1a")).into_node())
                .child_node(Text::new("Welcome to our service.")
                    .font_size(Px(16)).line_height(1.6).into_node())
                .child_node(Button::new("https://example.com", "Get Started")
                    .background(Color::hex("2563eb"))
                    .text_color(Color::white())
                    .border_radius(Px(8)).into_node())
        )))
    .into_node();
```

## Email Client Compatibility

All components render using table-based layouts where needed for Outlook compatibility. The `Button` component, for example, produces a `<table><td><a>` structure that renders correctly in Outlook's Word-based rendering engine.

## Re-exports

This crate re-exports all types from `ferrum-email-core` for convenience, so you typically only need to depend on `ferrum-email-components`.

## License

MIT OR Apache-2.0
