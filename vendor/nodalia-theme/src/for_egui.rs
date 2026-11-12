//! The NDL tokens mapped onto egui.
//!
//! egui has fewer style slots than NDL has tokens, so this is a mapping, not a one-to-one copy.
//! What egui cannot express (per-component radii, the focus ring offset, easing curves, font
//! weights) stays available as constants in the crate root for custom widgets.
//!
//! ```no_run
//! # let ctx = egui::Context::default();
//! use nodalia_theme::for_egui::{self, Appearance};
//! use nodalia_theme::Mode;
//!
//! // Follow the system light/dark preference:
//! for_egui::apply_system(&ctx, &Appearance::default());
//! // Or force one mode:
//! for_egui::apply(&ctx, &Appearance { mode: Mode::DarkHc, ..Appearance::default() });
//! ```

use ::egui::{
    style::{Selection, WidgetVisuals},
    Color32, Context, CornerRadius, FontId, Margin, Stroke, Style, TextStyle, Theme,
    ThemePreference, Vec2, Visuals,
};

use crate::{
    border_width, component, font_size, line_height, motion, radius, size, space, Accent, Density,
    Mode, Palette, Rgba, Shadow,
};

/// What the user (or the system) chose. `mode` is ignored by [`apply_system`] except for contrast.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Appearance {
    pub mode: Mode,
    /// The accent of the app. `Accent::Nodalia` unless the app has its own.
    pub accent: Accent,
    pub density: Density,
    /// Collapses animations to zero. Bind it to the system "reduce motion" preference.
    pub reduced_motion: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            mode: Mode::Light,
            accent: Accent::Nodalia,
            density: Density::REGULAR,
            reduced_motion: false,
        }
    }
}

/// An NDL colour as an egui colour.
pub fn color(value: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(value.r, value.g, value.b, value.a)
}

fn corner(value: f32) -> CornerRadius {
    CornerRadius::same(value.round().clamp(0.0, 255.0) as u8)
}

fn margin(value: f32) -> Margin {
    Margin::same(value.round().clamp(0.0, 127.0) as i8)
}

fn shadow(value: Shadow) -> ::egui::Shadow {
    let offset = |v: f32| v.round().clamp(-128.0, 127.0) as i8;
    let size = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    ::egui::Shadow {
        offset: [offset(value.offset_x), offset(value.offset_y)],
        blur: size(value.blur),
        spread: size(value.spread),
        color: color(value.color),
    }
}

fn widget(fill: Rgba, border: Rgba, border_width: f32, text: Rgba) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: color(fill),
        weak_bg_fill: color(fill),
        bg_stroke: Stroke::new(border_width, color(border)),
        corner_radius: corner(component::BUTTON_RADIUS),
        fg_stroke: Stroke::new(border_width::THIN, color(text)),
        expansion: 0.0,
    }
}

/// egui visuals for one mode with the default accent.
pub fn visuals(mode: Mode) -> Visuals {
    visuals_with(mode, Accent::Nodalia)
}

/// egui visuals for one mode and the accent of an app.
pub fn visuals_with(mode: Mode, accent: Accent) -> Visuals {
    let p: &Palette = &mode.palette_with(accent);
    let mut v = if mode.is_dark() {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.dark_mode = mode.is_dark();
    v.override_text_color = None;
    v.weak_text_color = Some(color(p.text_secondary));

    // Static content draws on a panel; controls stack the three control backgrounds.
    v.widgets.noninteractive = widget(
        p.surface_panel,
        p.border_subtle,
        border_width::THIN,
        p.text_primary,
    );
    v.widgets.inactive = widget(
        p.control_bg,
        p.border_default,
        border_width::THIN,
        p.text_primary,
    );
    v.widgets.hovered = widget(
        p.control_bg_hover,
        p.border_strong,
        border_width::THIN,
        p.text_primary,
    );
    v.widgets.active = widget(
        p.control_bg_pressed,
        p.border_focus,
        border_width::THIN,
        p.text_primary,
    );
    v.widgets.open = widget(
        p.control_bg_hover,
        p.border_default,
        border_width::THIN,
        p.text_primary,
    );

    // egui uses the selection stroke both for selected content and for the keyboard focus outline.
    v.selection = Selection {
        bg_fill: color(p.selection_bg),
        stroke: Stroke::new(border_width::THICK, color(p.accent_text)),
    };

    v.hyperlink_color = color(p.text_link);
    v.faint_bg_color = color(p.surface_sunken);
    v.extreme_bg_color = color(p.control_field_bg);
    v.text_edit_bg_color = Some(color(p.control_field_bg));
    v.code_bg_color = color(p.surface_sunken);
    v.warn_fg_color = color(p.status_warning_fg);
    v.error_fg_color = color(p.status_danger_fg);

    v.panel_fill = color(p.surface_panel);
    v.window_fill = color(p.surface_overlay);
    v.window_stroke = Stroke::new(border_width::THIN, color(p.border_subtle));
    v.window_corner_radius = corner(radius::LG);
    v.menu_corner_radius = corner(radius::MD);
    v.window_shadow = shadow(p.elevation_overlay);
    v.popup_shadow = shadow(p.elevation_raised);
    v.text_cursor.stroke = Stroke::new(border_width::THICK, color(p.text_primary));
    v
}

/// A full egui style: visuals, spacing, text sizes and animation time.
pub fn style(appearance: &Appearance) -> Style {
    let d = appearance.density;
    let mut s = Style {
        visuals: visuals_with(appearance.mode, appearance.accent),
        ..Style::default()
    };

    s.spacing.item_spacing = Vec2::splat(d.gap);
    s.spacing.button_padding = Vec2::new(
        d.control_padding_x,
        ((d.control_height - line_height::BODY) / 2.0).max(0.0),
    );
    s.spacing.interact_size.y = d.control_height.max(size::TARGET_MIN);
    s.spacing.window_margin = margin(d.panel_padding);
    s.spacing.menu_margin = margin(space::XS);
    s.spacing.indent = space::LG;
    s.spacing.icon_width = component::TOGGLE_BOX_SIZE;
    s.spacing.icon_spacing = space::SM;
    s.spacing.slider_rail_height = component::SLIDER_TRACK_HEIGHT;
    s.spacing.tooltip_width = size::TOOLTIP_MAX;
    s.spacing.scroll.bar_width = size::SCROLLBAR;

    // Only sizes are set: NDL does not ship fonts, so the families stay whatever the app loaded.
    for (text_style, font) in &mut s.text_styles {
        let family = font.family.clone();
        let px = match text_style {
            TextStyle::Small => font_size::SMALL,
            TextStyle::Body | TextStyle::Button => font_size::BODY,
            TextStyle::Monospace => font_size::SMALL,
            TextStyle::Heading => font_size::TITLE,
            _ => continue,
        };
        *font = FontId::new(px, family);
    }

    s.animation_time = motion::STATE.seconds(appearance.reduced_motion);
    s
}

/// Applies one fixed mode: sets the style and stops following the system theme.
pub fn apply(ctx: &Context, appearance: &Appearance) {
    let theme = if appearance.mode.is_dark() {
        Theme::Dark
    } else {
        Theme::Light
    };
    ctx.set_style_of(theme, style(appearance));
    ctx.set_theme(theme);
}

/// Follows the system light/dark preference. `appearance.mode` only decides the contrast:
/// a high-contrast mode installs the two high-contrast palettes.
pub fn apply_system(ctx: &Context, appearance: &Appearance) {
    let (light, dark) = if appearance.mode.is_high_contrast() {
        (Mode::LightHc, Mode::DarkHc)
    } else {
        (Mode::Light, Mode::Dark)
    };
    ctx.set_style_of(
        Theme::Light,
        style(&Appearance {
            mode: light,
            ..*appearance
        }),
    );
    ctx.set_style_of(
        Theme::Dark,
        style(&Appearance {
            mode: dark,
            ..*appearance
        }),
    );
    ctx.set_theme(ThemePreference::System);
}
