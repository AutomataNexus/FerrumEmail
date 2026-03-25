# ferrum-email-core

The foundation crate for the [Ferrum Email](https://github.com/AutomataNexus/ferrum-email) framework.

## What's Inside

- **`Component` trait** — the core abstraction. Every email template and reusable element implements this.
- **`Node` tree** — the intermediate representation between components and rendered HTML. Consists of `Element`, `Text`, `Fragment`, and `None` variants.
- **`Style` system** — type-safe CSS properties (`color`, `font-size`, `padding`, etc.) with a `to_css()` method for inline style generation.
- **`Color`** — hex, RGB, RGBA, named, and transparent color values.
- **`Spacing`** — padding/margin with `all()`, `xy()`, `new()` constructors.
- **Primitive types** — `Px`, `Percent`, `SizeValue`, `FontWeight`, `TextAlign`, `VerticalAlign`, `FontFamily`, `LineHeight`, `Display`, `BorderStyle`, `TextDecoration`, `HeadingLevel`.

## Usage

```rust
use ferrum_email_core::{Component, Node, Element, Tag, Style, Color, Px};

struct MyComponent;

impl Component for MyComponent {
    fn render(&self) -> Node {
        let mut style = Style::new();
        style.color = Some(Color::hex("333333"));
        style.font_size = Some(Px(16));

        Node::Element(
            Element::new(Tag::P)
                .style(style)
                .child(Node::text("Hello from Ferrum!")),
        )
    }
}
```

## Design

The `Node` tree is intentionally simple — it's a direct mapping to HTML elements with typed attributes and styles. This keeps the renderer straightforward while giving components full control over their output.

All style values are type-checked at compile time. You cannot construct invalid CSS values — `Px(16)` renders to `"16px"`, `Color::hex("ff0000")` renders to `"#ff0000"`, etc.

## License

MIT OR Apache-2.0
