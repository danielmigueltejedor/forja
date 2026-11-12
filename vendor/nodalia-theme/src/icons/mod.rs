//! NDL's icon set: every icon's SVG elements on the 24 x 24 grid, generated from
//! `packages/icons/src` so Rust apps draw the same shapes as the web, GTK and Qt.
//!
//! Each [`Icon`] has its regular body (the 1.75 [`STROKE`], outlines), its bold body (the
//! [`BOLD_STROKE`], with NDL's solid parts filled) and the parts a CAD app draws in its accent
//! colour. With the `egui` feature, [`paint`] draws one into an `egui::Painter`.
//!
//! ```
//! let line = nodalia_theme::icons::get("line").expect("NDL has a line icon");
//! assert_eq!(line.group, "sketch");
//! assert!(nodalia_theme::icons::get("no-such-icon").is_none());
//! ```

#[rustfmt::skip]
mod data;

#[cfg(feature = "egui")]
mod paint;
#[cfg(feature = "egui")]
pub use paint::{paint, Style};

/// The regular stroke on the 24 grid.
pub const STROKE: f32 = 1.75;
/// The bold stroke on the 24 grid.
pub const BOLD_STROKE: f32 = 2.25;

/// One NDL icon.
#[derive(Debug, PartialEq, Eq)]
pub struct Icon {
    /// Its kebab-case name, the same as the SVG file's.
    pub name: &'static str,
    /// The group it is listed under (`edit`, `sketch`, `cad`, ...).
    pub group: &'static str,
    /// The regular style's SVG elements.
    pub body: &'static [&'static str],
    /// The bold style's SVG elements when they differ from `body`, else empty: use
    /// [`Icon::bold_body`].
    pub bold: &'static [&'static str],
    /// The indices of the elements a CAD app draws in its accent colour.
    pub accent: &'static [usize],
}

impl Icon {
    /// The bold style's SVG elements.
    pub fn bold_body(&self) -> &'static [&'static str] {
        if self.bold.is_empty() {
            self.body
        } else {
            self.bold
        }
    }
}

/// Every icon, sorted by name.
pub fn all() -> &'static [Icon] {
    &data::ICONS
}

/// The icon called `name`.
pub fn get(name: &str) -> Option<&'static Icon> {
    let icons = all();
    icons
        .binary_search_by(|i| i.name.cmp(name))
        .ok()
        .and_then(|i| icons.get(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_sorted_and_unique() {
        assert!(all().windows(2).all(|w| w[0].name < w[1].name));
        assert!(all().len() > 300);
    }

    #[test]
    fn every_icon_can_be_found_and_its_accents_exist() {
        for icon in all() {
            assert_eq!(get(icon.name), Some(icon));
            assert!(
                !icon.body.is_empty() && !icon.bold_body().is_empty(),
                "{}",
                icon.name
            );
            assert!(
                icon.accent.iter().all(|&i| i < icon.body.len()),
                "{}",
                icon.name
            );
        }
    }

    #[test]
    fn the_cad_set_is_there() {
        for name in ["line", "trim", "dim-continue", "layer-freeze", "coincident"] {
            assert!(get(name).is_some(), "{name}");
        }
        assert!(all().iter().filter(|i| i.group == "cad").count() >= 93);
    }
}
