//! Spacing type for padding and margin values.

use crate::types::Px;
use std::fmt;

/// Spacing values for padding and margin (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    pub top: Px,
    pub right: Px,
    pub bottom: Px,
    pub left: Px,
}

impl Spacing {
    /// All four sides the same.
    pub fn all(value: Px) -> Self {
        Spacing {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Vertical and horizontal (top/bottom, left/right).
    pub fn xy(vertical: Px, horizontal: Px) -> Self {
        Spacing {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Each side specified individually.
    pub fn new(top: Px, right: Px, bottom: Px, left: Px) -> Self {
        Spacing {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Zero spacing on all sides.
    pub fn zero() -> Self {
        Spacing::all(Px(0))
    }
}

impl fmt::Display for Spacing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.top == self.right && self.right == self.bottom && self.bottom == self.left {
            write!(f, "{}", self.top)
        } else if self.top == self.bottom && self.left == self.right {
            write!(f, "{} {}", self.top, self.right)
        } else {
            write!(
                f,
                "{} {} {} {}",
                self.top, self.right, self.bottom, self.left
            )
        }
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Spacing::zero()
    }
}
