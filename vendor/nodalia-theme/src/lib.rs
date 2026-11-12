//! Nodalia Design Language (NDL) for Rust.
//!
//! The tokens are generated from the DTCG sources of `@nodalia/tokens`; nothing here is typed by
//! hand except the mapping onto egui. Without features the crate has no dependencies and can be
//! used from any toolkit. With the `egui` feature, [`for_egui`] builds an `egui::Style`.
//!
//! ```
//! use nodalia_theme::{Density, Mode};
//!
//! let palette = Mode::Dark.palette();
//! assert!(nodalia_theme::contrast(palette.text_primary, palette.surface_panel) >= 4.5);
//! assert_eq!(Density::REGULAR.control_height, 32.0);
//! ```

// Generated file: keep it byte-identical to the generator's output.
#[rustfmt::skip]
mod tokens;

pub use tokens::*;

#[cfg(feature = "egui")]
pub mod for_egui;
pub mod icons;

impl Density {
    pub const ALL: [Density; 3] = [Density::COMPACT, Density::REGULAR, Density::COMFORTABLE];
}

impl Default for Density {
    fn default() -> Self {
        Density::REGULAR
    }
}

impl Transition {
    /// Duration in seconds, or zero when the user asked for reduced motion.
    pub fn seconds(self, reduced_motion: bool) -> f32 {
        if reduced_motion {
            0.0
        } else {
            self.duration_ms as f32 / 1000.0
        }
    }
}

fn linear(channel: u8) -> f64 {
    let s = f64::from(channel) / 255.0;
    if s <= 0.040_45 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.x relative luminance of an opaque colour (alpha is ignored).
pub fn luminance(color: Rgba) -> f64 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// WCAG 2.x contrast ratio between two opaque colours, from 1 to 21.
pub fn contrast(foreground: Rgba, background: Rgba) -> f64 {
    let a = luminance(foreground);
    let b = luminance(background);
    let (high, low) = if a > b { (a, b) } else { (b, a) };
    (high + 0.05) / (low + 0.05)
}
