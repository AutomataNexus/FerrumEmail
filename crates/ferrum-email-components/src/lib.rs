//! # ferrum-email-components
//!
//! Standard component library for the Ferrum Email framework.
//!
//! Provides email-safe, composable components for building email templates:
//!
//! - **Layout**: `Html`, `Head`, `Body`, `Container`, `Section`, `Row`, `Column`
//! - **Content**: `Text`, `Heading`, `Link`, `Image`, `Button`
//! - **Utility**: `Preview`, `Hr`, `Spacer`, `Code`, `CodeBlock`
//!
//! All components implement the `Component` trait from `ferrum_email_core` and produce
//! email-safe HTML node trees using table-based layouts for maximum compatibility.

pub mod body;
pub mod button;
pub mod code;
pub mod column;
pub mod container;
pub mod head;
pub mod heading;
pub mod hr;
pub mod html;
pub mod image;
pub mod link;
pub mod preview;
pub mod row;
pub mod section;
pub mod spacer;
pub mod text;

pub use body::Body;
pub use button::Button;
pub use code::{Code, CodeBlock};
pub use column::Column;
pub use container::Container;
pub use head::Head;
pub use heading::Heading;
pub use hr::Hr;
pub use html::Html;
pub use image::Image;
pub use link::Link;
pub use preview::Preview;
pub use row::Row;
pub use section::Section;
pub use spacer::Spacer;
pub use text::Text;

// Re-export core types for convenience — users can import everything from ferrum_email_components.
pub use ferrum_email_core::{
    Border, BorderStyle, Color, Component, Display, FontFamily, FontWeight, HeadingLevel,
    LineHeight, Node, Percent, Px, SizeValue, Spacing, Style, TextAlign, TextDecoration,
    VerticalAlign,
};
