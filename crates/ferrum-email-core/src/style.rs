//! Typed CSS style system for email-safe styling.
//!
//! All style values are type-checked at compile time. The style system only includes
//! properties that are safe to use across email clients.

use crate::color::Color;
use crate::spacing::Spacing;
use crate::types::*;

/// A typed inline style map for email elements.
///
/// All properties are optional. Only set properties will be emitted as CSS.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub font_family: Option<FontFamily>,
    pub font_size: Option<Px>,
    pub font_weight: Option<FontWeight>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub padding: Option<Spacing>,
    pub margin: Option<Spacing>,
    pub border_radius: Option<Px>,
    pub width: Option<SizeValue>,
    pub max_width: Option<SizeValue>,
    pub min_width: Option<SizeValue>,
    pub height: Option<SizeValue>,
    pub text_align: Option<TextAlign>,
    pub vertical_align: Option<VerticalAlign>,
    pub line_height: Option<LineHeight>,
    pub display: Option<Display>,
    pub border: Option<Border>,
    pub border_bottom: Option<Border>,
    pub border_top: Option<Border>,
    pub text_decoration: Option<TextDecoration>,
    pub letter_spacing: Option<Px>,
    pub word_spacing: Option<Px>,
}

/// A border specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub width: Px,
    pub style: BorderStyle,
    pub color: Color,
}

impl Border {
    pub fn new(width: Px, style: BorderStyle, color: Color) -> Self {
        Border {
            width,
            style,
            color,
        }
    }

    pub fn solid(width: Px, color: Color) -> Self {
        Border::new(width, BorderStyle::Solid, color)
    }
}

impl std::fmt::Display for Border {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.width, self.style, self.color)
    }
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render this style to a CSS inline style string.
    ///
    /// Returns `None` if no properties are set.
    pub fn to_css(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(ref ff) = self.font_family {
            parts.push(format!("font-family:{ff}"));
        }
        if let Some(ref fs) = self.font_size {
            parts.push(format!("font-size:{fs}"));
        }
        if let Some(ref fw) = self.font_weight {
            parts.push(format!("font-weight:{fw}"));
        }
        if let Some(ref c) = self.color {
            parts.push(format!("color:{c}"));
        }
        if let Some(ref bg) = self.background_color {
            parts.push(format!("background-color:{bg}"));
        }
        if let Some(ref p) = self.padding {
            parts.push(format!("padding:{p}"));
        }
        if let Some(ref m) = self.margin {
            parts.push(format!("margin:{m}"));
        }
        if let Some(ref br) = self.border_radius {
            parts.push(format!("border-radius:{br}"));
        }
        if let Some(ref w) = self.width {
            parts.push(format!("width:{w}"));
        }
        if let Some(ref mw) = self.max_width {
            parts.push(format!("max-width:{mw}"));
        }
        if let Some(ref mw) = self.min_width {
            parts.push(format!("min-width:{mw}"));
        }
        if let Some(ref h) = self.height {
            parts.push(format!("height:{h}"));
        }
        if let Some(ref ta) = self.text_align {
            parts.push(format!("text-align:{ta}"));
        }
        if let Some(ref va) = self.vertical_align {
            parts.push(format!("vertical-align:{va}"));
        }
        if let Some(ref lh) = self.line_height {
            parts.push(format!("line-height:{lh}"));
        }
        if let Some(ref d) = self.display {
            parts.push(format!("display:{d}"));
        }
        if let Some(ref b) = self.border {
            parts.push(format!("border:{b}"));
        }
        if let Some(ref b) = self.border_bottom {
            parts.push(format!("border-bottom:{b}"));
        }
        if let Some(ref b) = self.border_top {
            parts.push(format!("border-top:{b}"));
        }
        if let Some(ref td) = self.text_decoration {
            parts.push(format!("text-decoration:{td}"));
        }
        if let Some(ref ls) = self.letter_spacing {
            parts.push(format!("letter-spacing:{ls}"));
        }
        if let Some(ref ws) = self.word_spacing {
            parts.push(format!("word-spacing:{ws}"));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(";"))
        }
    }

    /// Merge another style into this one. Properties from `other` override.
    pub fn merge(&mut self, other: &Style) {
        macro_rules! merge_field {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        merge_field!(font_family);
        merge_field!(font_size);
        merge_field!(font_weight);
        merge_field!(color);
        merge_field!(background_color);
        merge_field!(padding);
        merge_field!(margin);
        merge_field!(border_radius);
        merge_field!(width);
        merge_field!(max_width);
        merge_field!(min_width);
        merge_field!(height);
        merge_field!(text_align);
        merge_field!(vertical_align);
        merge_field!(line_height);
        merge_field!(display);
        merge_field!(border);
        merge_field!(border_bottom);
        merge_field!(border_top);
        merge_field!(text_decoration);
        merge_field!(letter_spacing);
        merge_field!(word_spacing);
    }
}
