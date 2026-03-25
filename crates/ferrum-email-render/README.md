# ferrum-email-render

The rendering engine for [Ferrum Email](https://github.com/AutomataNexus/ferrum-email).

## What It Does

Takes a `Component`, calls its `render()` method, and produces:

1. **Email-safe HTML** — all CSS inlined as `style=""` attributes, HTML entities escaped, void elements self-closed
2. **Plain text** — extracted from the node tree with links as `text (url)`, images as `[alt]`, and horizontal rules as `---`

## Pipeline

```
Component::render() → Node tree
       ↓
   CSS Inliner       (Style structs → style="" attributes)
       ↓
   HTML Emitter      (Node tree → HTML string)
       ↓
   Text Extractor    (Node tree → plain text fallback)
```

## Usage

```rust
use ferrum_email_render::Renderer;
use ferrum_email_core::Component;

let renderer = Renderer::default();

let html = renderer.render_html(&my_email).unwrap();
let text = renderer.render_text(&my_email).unwrap();
```

### Configuration

```rust
use ferrum_email_render::{Renderer, RenderConfig};

let renderer = Renderer::with_config(RenderConfig {
    include_doctype: true,     // Prepend <!DOCTYPE html>
    pretty_print: false,       // Compact output (default)
    indent: "  ".to_string(),  // Indentation for pretty-print
});
```

## Key Decisions

- **No `<style>` blocks** — Gmail strips them. Everything is inlined.
- **DOCTYPE included by default** — `<!DOCTYPE html>` ensures standards mode.
- **Void elements** are self-closing: `<img />`, `<hr />`, `<br />`, `<meta />`.
- **Text escaping** handles `&`, `<`, `>` in content and additionally `"`, `'` in attributes.

## License

MIT OR Apache-2.0
