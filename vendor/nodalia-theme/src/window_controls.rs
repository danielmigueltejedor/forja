//! Client-drawn Nodalia window buttons. The app owns placement and window policy.

use egui::{vec2, CursorIcon, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo};

use crate::{border_width, component, for_egui::color, icons, space, Palette};

/// The space occupied by the three window controls, in logical points.
pub const WIDTH: f32 = 3.0 * component::WINDOW_CONTROL_SIZE + 2.0 * space::XS;

/// Localized accessibility labels for the window buttons.
pub struct Labels<'a> {
    pub minimize: &'a str,
    pub maximize: &'a str,
    pub restore: &'a str,
    pub close: &'a str,
}

/// An action for the app to perform on its window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl Action {
    fn icon(self) -> &'static str {
        match self {
            Self::Minimize => "window-minimize",
            Self::Maximize => "window-maximize",
            Self::Restore => "window-restore",
            Self::Close => "window-close",
        }
    }
}

/// Draw the three buttons at the right of `rect`, centered vertically. The app should reserve
/// at least [`WIDTH`] and [`component::WINDOW_CONTROL_SIZE`] for this region. The buttons
/// report the requested action, leaving viewport commands and close confirmation to the app.
pub fn show(
    ui: &Ui,
    rect: Rect,
    palette: &Palette,
    focused: bool,
    maximized: bool,
    labels: Labels<'_>,
) -> Option<Action> {
    let size = component::WINDOW_CONTROL_SIZE;
    let start = egui::pos2(rect.right() - WIDTH, rect.center().y - size / 2.0);
    let (zoom, zoom_label) = if maximized {
        (Action::Restore, labels.restore)
    } else {
        (Action::Maximize, labels.maximize)
    };

    let mut clicked = None;
    for (index, (action, label)) in [
        (Action::Minimize, labels.minimize),
        (zoom, zoom_label),
        (Action::Close, labels.close),
    ]
    .into_iter()
    .enumerate()
    {
        let bounds = Rect::from_min_size(
            start + vec2(index as f32 * (size + space::XS), 0.0),
            vec2(size, size),
        );
        // Maximize and Restore are two labels for the same control. Keep keyboard focus on it.
        let response = ui
            .interact(
                bounds,
                ui.scope_id().with(("ndl_window_control", index)),
                Sense::click(),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
        paint(ui, &response, action, palette, focused);
        if response.clicked() {
            clicked = Some(action);
        }
    }
    clicked
}

fn paint(ui: &Ui, response: &Response, action: Action, palette: &Palette, focused: bool) {
    let (fill, ink) = if !response.enabled() {
        (palette.control_bg_disabled, palette.text_disabled)
    } else if response.is_pointer_button_down_on() {
        (palette.control_bg_pressed, palette.text_primary)
    } else if response.hovered() {
        match action {
            Action::Minimize => (palette.window_minimize_tint, palette.window_minimize),
            Action::Maximize | Action::Restore => {
                (palette.window_maximize_tint, palette.window_maximize)
            }
            Action::Close => (palette.window_close, palette.window_close_icon),
        }
    } else if focused {
        (palette.control_bg, palette.text_primary)
    } else {
        (palette.control_bg, palette.text_secondary)
    };
    let center = response.rect.center();
    let radius = component::WINDOW_CONTROL_SIZE / 2.0;
    ui.painter().circle_filled(center, radius, color(fill));
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            radius,
            Stroke::new(border_width::THICK, color(palette.border_focus)),
            StrokeKind::Inside,
        );
    }
    if let Some(icon) = icons::get(action.icon()) {
        let glyph = Rect::from_center_size(
            center,
            vec2(
                component::WINDOW_CONTROL_ICON,
                component::WINDOW_CONTROL_ICON,
            ),
        );
        icons::paint(ui.painter(), glyph, icon, icons::Style::bold(color(ink)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, Context, Event, Modifiers, PointerButton, RawInput};

    fn frame(ctx: &Context, events: Vec<Event>, maximized: bool) -> Option<Action> {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(WIDTH, component::WINDOW_CONTROL_SIZE));
        let mut clicked = None;
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(rect),
                events,
                ..Default::default()
            },
            |ui| {
                clicked = show(
                    ui,
                    rect,
                    crate::Mode::Light.palette(),
                    true,
                    maximized,
                    Labels {
                        minimize: "Minimize",
                        maximize: "Maximize",
                        restore: "Restore",
                        close: "Close",
                    },
                );
            },
        );
        output.textures_delta.clear();
        clicked
    }

    #[test]
    fn clicks_select_the_control_and_report_the_current_zoom_action() {
        let size = component::WINDOW_CONTROL_SIZE;
        assert_eq!(size, 28.0);
        for (index, maximized, action) in [
            (0, false, Action::Minimize),
            (1, false, Action::Maximize),
            (1, true, Action::Restore),
            (2, true, Action::Close),
        ] {
            let ctx = Context::default();
            let pos = pos2(size / 2.0 + index as f32 * (size + space::XS), size / 2.0);
            assert!(icons::get(action.icon()).is_some());
            assert_eq!(frame(&ctx, vec![Event::PointerMoved(pos)], maximized), None);
            for pressed in [true, false] {
                let event = Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                };
                assert_eq!(
                    frame(&ctx, vec![event], maximized),
                    (!pressed).then_some(action)
                );
            }
        }
    }
}
