//! Ultra-light warm color theme for the Ferrum Email TUI.
//!
//! Cream, warm terracotta, ultra-light teal, sandy greys —
//! matching the Ferrum Email brand palette.
#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};

// ── Brand Colors ──────────────────────────────────────────

/// Warm off-white background
pub const BG: Color = Color::Rgb(250, 250, 248); // #FAFAF8
/// Slightly warmer card background
pub const CARD_BG: Color = Color::Rgb(255, 254, 250); // #FFFEFA
/// Footer / subtle section background
pub const SUBTLE_BG: Color = Color::Rgb(250, 248, 245); // #FAF8F5

/// Primary text — warm charcoal
pub const TEXT: Color = Color::Rgb(45, 42, 38); // #2D2A26
/// Secondary text — warm grey
pub const TEXT_MUTED: Color = Color::Rgb(74, 69, 64); // #4A4540
/// Tertiary text — sandy
pub const TEXT_DIM: Color = Color::Rgb(168, 153, 140); // #A8998C
/// Label text — warm brown
pub const LABEL: Color = Color::Rgb(139, 111, 94); // #8B6F5E

/// Brand terracotta — primary accent
pub const TERRACOTTA: Color = Color::Rgb(192, 88, 43); // #C0582B
/// Lighter terracotta for highlights
pub const TERRACOTTA_LIGHT: Color = Color::Rgb(214, 139, 100); // #D68B64
/// Rust red from logo
pub const RUST_RED: Color = Color::Rgb(192, 57, 43); // #C0392B

/// Ultra-light teal accent
pub const TEAL: Color = Color::Rgb(143, 188, 183); // #8FBCB7
/// Lighter teal for borders
pub const TEAL_LIGHT: Color = Color::Rgb(200, 224, 221); // #C8E0DD

/// Divider / border color — sandy
pub const BORDER: Color = Color::Rgb(232, 221, 212); // #E8DDD4
/// Stronger border
pub const BORDER_STRONG: Color = Color::Rgb(200, 187, 175); // #C8BBAF

/// Success green (muted)
pub const SUCCESS: Color = Color::Rgb(120, 160, 120); // #78A078
/// Error (warm red)
pub const ERROR: Color = Color::Rgb(180, 70, 60); // #B4463C
/// Warning (warm amber)
pub const WARNING: Color = Color::Rgb(200, 160, 80); // #C8A050

// ── Style Presets ─────────────────────────────────────────

pub fn title() -> Style {
    Style::default().fg(TERRACOTTA).add_modifier(Modifier::BOLD)
}

pub fn title_secondary() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

pub fn text_normal() -> Style {
    Style::default().fg(TEXT)
}

pub fn text_muted() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn text_dim() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn label() -> Style {
    Style::default().fg(LABEL).add_modifier(Modifier::BOLD)
}

pub fn highlight() -> Style {
    Style::default()
        .fg(CARD_BG)
        .bg(TERRACOTTA)
        .add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().fg(TERRACOTTA).add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER_STRONG)
}

pub fn tab_active() -> Style {
    Style::default()
        .fg(TERRACOTTA)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn tab_inactive() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn status_ok() -> Style {
    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)
}

pub fn status_err() -> Style {
    Style::default().fg(ERROR).add_modifier(Modifier::BOLD)
}

pub fn keybind() -> Style {
    Style::default().fg(TEAL).add_modifier(Modifier::BOLD)
}

pub fn keybind_desc() -> Style {
    Style::default().fg(TEXT_DIM)
}
