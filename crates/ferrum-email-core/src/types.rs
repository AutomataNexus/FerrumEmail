//! Primitive types used throughout the Ferrum Email framework.
//!
//! These types provide compile-time safety for CSS values commonly used in email templates.

use std::fmt;

/// A pixel value for sizing and spacing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Px(pub u32);

impl fmt::Display for Px {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.0)
    }
}

impl From<u32> for Px {
    fn from(val: u32) -> Self {
        Px(val)
    }
}

/// A percentage value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(pub f32);

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// A size value that can be either pixels or a percentage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeValue {
    Px(Px),
    Percent(Percent),
    Auto,
}

impl fmt::Display for SizeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeValue::Px(px) => write!(f, "{px}"),
            SizeValue::Percent(pct) => write!(f, "{pct}"),
            SizeValue::Auto => write!(f, "auto"),
        }
    }
}

impl From<Px> for SizeValue {
    fn from(px: Px) -> Self {
        SizeValue::Px(px)
    }
}

impl From<Percent> for SizeValue {
    fn from(pct: Percent) -> Self {
        SizeValue::Percent(pct)
    }
}

/// Font weight values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Numeric(u16),
}

impl fmt::Display for FontWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontWeight::Thin => write!(f, "100"),
            FontWeight::Light => write!(f, "300"),
            FontWeight::Normal => write!(f, "400"),
            FontWeight::Medium => write!(f, "500"),
            FontWeight::SemiBold => write!(f, "600"),
            FontWeight::Bold => write!(f, "700"),
            FontWeight::ExtraBold => write!(f, "800"),
            FontWeight::Black => write!(f, "900"),
            FontWeight::Numeric(n) => write!(f, "{n}"),
        }
    }
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl fmt::Display for TextAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextAlign::Left => write!(f, "left"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::Right => write!(f, "right"),
        }
    }
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

impl fmt::Display for VerticalAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerticalAlign::Top => write!(f, "top"),
            VerticalAlign::Middle => write!(f, "middle"),
            VerticalAlign::Bottom => write!(f, "bottom"),
        }
    }
}

/// Font family specification.
#[derive(Debug, Clone, PartialEq)]
pub enum FontFamily {
    /// A named font stack (e.g., "Arial, Helvetica, sans-serif").
    Named(String),
    /// Common email-safe font stacks.
    SansSerif,
    Serif,
    Monospace,
}

impl fmt::Display for FontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontFamily::Named(name) => write!(f, "{name}"),
            FontFamily::SansSerif => {
                write!(f, "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif")
            }
            FontFamily::Serif => write!(f, "Georgia, 'Times New Roman', Times, serif"),
            FontFamily::Monospace => {
                write!(f, "'SFMono-Regular', Menlo, Consolas, 'Courier New', monospace")
            }
        }
    }
}

/// Line height value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// Unitless multiplier (e.g., 1.5).
    Multiplier(f32),
    /// Fixed pixel value.
    Px(Px),
}

impl fmt::Display for LineHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineHeight::Multiplier(m) => write!(f, "{m}"),
            LineHeight::Px(px) => write!(f, "{px}"),
        }
    }
}

impl From<f32> for LineHeight {
    fn from(val: f32) -> Self {
        LineHeight::Multiplier(val)
    }
}

/// Display property values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Block,
    InlineBlock,
    Inline,
    None,
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Display::Block => write!(f, "block"),
            Display::InlineBlock => write!(f, "inline-block"),
            Display::Inline => write!(f, "inline"),
            Display::None => write!(f, "none"),
        }
    }
}

/// Border style values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
}

impl fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorderStyle::None => write!(f, "none"),
            BorderStyle::Solid => write!(f, "solid"),
            BorderStyle::Dashed => write!(f, "dashed"),
            BorderStyle::Dotted => write!(f, "dotted"),
        }
    }
}

/// Text decoration values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
    LineThrough,
}

impl fmt::Display for TextDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextDecoration::None => write!(f, "none"),
            TextDecoration::Underline => write!(f, "underline"),
            TextDecoration::LineThrough => write!(f, "line-through"),
        }
    }
}

/// Heading level (H1-H6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    /// Returns the HTML tag name for this heading level.
    pub fn tag_name(&self) -> &'static str {
        match self {
            HeadingLevel::H1 => "h1",
            HeadingLevel::H2 => "h2",
            HeadingLevel::H3 => "h3",
            HeadingLevel::H4 => "h4",
            HeadingLevel::H5 => "h5",
            HeadingLevel::H6 => "h6",
        }
    }

    /// Returns the default font size for this heading level.
    pub fn default_font_size(&self) -> Px {
        match self {
            HeadingLevel::H1 => Px(32),
            HeadingLevel::H2 => Px(28),
            HeadingLevel::H3 => Px(24),
            HeadingLevel::H4 => Px(20),
            HeadingLevel::H5 => Px(16),
            HeadingLevel::H6 => Px(14),
        }
    }
}
