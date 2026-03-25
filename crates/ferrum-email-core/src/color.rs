//! Color type for the Ferrum Email framework.
//!
//! Provides a type-safe color representation supporting hex, RGB, and named colors.

use std::fmt;

/// A color value for use in email styles.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    /// Hex color (stored without the `#` prefix).
    Hex(String),
    /// RGB color.
    Rgb(u8, u8, u8),
    /// RGBA color with alpha (0.0–1.0).
    Rgba(u8, u8, u8, f32),
    /// A named CSS color.
    Named(String),
    /// Transparent.
    Transparent,
}

impl Color {
    /// Create a color from a hex string (with or without `#`).
    pub fn hex(hex: &str) -> Self {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        Color::Hex(hex.to_string())
    }

    /// Create a color from RGB values.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb(r, g, b)
    }

    /// Create a color from RGBA values.
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Color::Rgba(r, g, b, a)
    }

    /// White (#ffffff).
    pub fn white() -> Self {
        Color::Hex("ffffff".to_string())
    }

    /// Black (#000000).
    pub fn black() -> Self {
        Color::Hex("000000".to_string())
    }

    /// Transparent.
    pub fn transparent() -> Self {
        Color::Transparent
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Hex(hex) => write!(f, "#{hex}"),
            Color::Rgb(r, g, b) => write!(f, "rgb({r},{g},{b})"),
            Color::Rgba(r, g, b, a) => write!(f, "rgba({r},{g},{b},{a})"),
            Color::Named(name) => write!(f, "{name}"),
            Color::Transparent => write!(f, "transparent"),
        }
    }
}
