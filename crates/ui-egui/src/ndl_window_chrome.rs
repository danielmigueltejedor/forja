//! The undecorated Linux root viewport. Drawing and CAD input stay inside the resize bands.

use egui::{CursorIcon, PointerButton, Rect, ResizeDirection, Sense, Ui, ViewportCommand, pos2};
use nodalia_theme::{component, space, window_controls};

use crate::theme;

pub(crate) const TITLE_HEIGHT: f32 = component::WINDOW_CONTROL_SIZE + 2.0 * space::XS;
pub(crate) const RESIZE_INSET: i8 = space::XS as i8;

pub(crate) fn title_row(ui: &mut Ui, row: Rect, title: String) {
    let (focused, maximized) = ui.input(|i| (i.focused, i.viewport().maximized.unwrap_or(false)));
    let buttons_right = row.right() - space::SM;
    let buttons_left = buttons_right - window_controls::WIDTH;
    let buttons = Rect::from_min_max(pos2(buttons_left, row.top()), pos2(buttons_right, row.bottom()));
    let drag_area = row.with_max_x(buttons_left);
    // The controls and the empty title area have disjoint hit regions.
    if drag_area.is_positive() {
        craft_window::title_bar::empty_area(ui, drag_area);
        let clip = drag_area.shrink2(egui::vec2(space::SM, 0.0));
        let painter = ui.painter().with_clip_rect(clip);
        let ink = theme::Tokens::get().text_dim;
        let galley = painter.layout_no_wrap(title, theme::body(), ink);
        let x = (row.center().x - galley.size().x / 2.0).clamp(clip.left(), (clip.right() - galley.size().x).max(clip.left()));
        let y = row.center().y - galley.size().y / 2.0;
        painter.galley(pos2(x, y), galley, ink);
    }
    let labels = window_controls::Labels {
        minimize: crate::tl!("Minimize"),
        maximize: crate::tl!("Maximize"),
        restore: crate::tl!("Restore"),
        close: crate::tl!("Close"),
    };
    if let Some(action) = window_controls::show(ui, buttons, &theme::ndl_palette(), focused, maximized, labels) {
        ui.ctx().send_viewport_cmd(viewport_command(action));
    }
}

fn viewport_command(action: window_controls::Action) -> ViewportCommand {
    use window_controls::Action;
    match action {
        Action::Minimize => ViewportCommand::Minimized(true),
        Action::Maximize => ViewportCommand::Maximized(true),
        Action::Restore => ViewportCommand::Maximized(false),
        // eframe delivers this through the same close-request path as a native window close.
        Action::Close => ViewportCommand::Close,
    }
}

pub(crate) fn can_resize(ctx: &egui::Context) -> bool {
    ctx.input(|i| !i.viewport().maximized.unwrap_or(false) && !i.viewport().fullscreen.unwrap_or(false))
}

pub(crate) fn resize_edges(ui: &Ui, full: Rect) {
    for (index, (direction, cursor, rect)) in resize_bands(full).into_iter().enumerate() {
        if !rect.is_positive() {
            continue;
        }
        let response = ui.interact(rect, ui.scope_id().with(("ndl_resize", index)), Sense::drag()).on_hover_cursor(cursor);
        // Drag-only egui responses start on button press, while winit has the Wayland grab serial.
        if response.drag_started_by(PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(direction));
        }
    }
}

/// Corner bands run along the two adjoining edges, leaving the drawing area clear.
fn resize_bands(full: Rect) -> [(ResizeDirection, CursorIcon, Rect); 12] {
    use {CursorIcon as C, ResizeDirection as D};
    let (l, t, r, b) = (full.left(), full.top(), full.right(), full.bottom());
    let e = f32::from(RESIZE_INSET).min(full.width() / 2.0).min(full.height() / 2.0);
    let x = space::LG.min(full.width() / 2.0);
    let y = space::LG.min(full.height() / 2.0);
    let rect = |left, top, right, bottom| Rect::from_min_max(pos2(left, top), pos2(right, bottom));
    [
        (D::North, C::ResizeNorth, rect(l + x, t, r - x, t + e)),
        (D::South, C::ResizeSouth, rect(l + x, b - e, r - x, b)),
        (D::West, C::ResizeWest, rect(l, t + y, l + e, b - y)),
        (D::East, C::ResizeEast, rect(r - e, t + y, r, b - y)),
        (D::NorthWest, C::ResizeNorthWest, rect(l, t, l + x, t + e)),
        (D::NorthWest, C::ResizeNorthWest, rect(l, t + e, l + e, t + y)),
        (D::NorthEast, C::ResizeNorthEast, rect(r - x, t, r, t + e)),
        (D::NorthEast, C::ResizeNorthEast, rect(r - e, t + e, r, t + y)),
        (D::SouthWest, C::ResizeSouthWest, rect(l, b - e, l + x, b)),
        (D::SouthWest, C::ResizeSouthWest, rect(l, b - y, l + e, b - e)),
        (D::SouthEast, C::ResizeSouthEast, rect(r - x, b - e, r, b)),
        (D::SouthEast, C::ResizeSouthEast, rect(r - e, b - y, r, b - e)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_request_the_standard_viewport_actions() {
        use window_controls::Action;
        assert_eq!(viewport_command(Action::Minimize), ViewportCommand::Minimized(true));
        assert_eq!(viewport_command(Action::Maximize), ViewportCommand::Maximized(true));
        assert_eq!(viewport_command(Action::Restore), ViewportCommand::Maximized(false));
        assert_eq!(viewport_command(Action::Close), ViewportCommand::Close);
    }

    fn input_frame(ctx: &egui::Context, time: f64, events: Vec<egui::Event>, maximized: bool) -> Vec<ViewportCommand> {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(600.0, 400.0));
        let mut input = egui::RawInput { screen_rect: Some(rect), time: Some(time), events, ..Default::default() };
        if let Some(root) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
            root.maximized = Some(maximized);
        }
        let mut output = ctx.run_ui(input, |ui| {
            let row = rect.shrink(f32::from(RESIZE_INSET)).with_max_y(TITLE_HEIGHT + f32::from(RESIZE_INSET));
            title_row(ui, row, String::new());
            if can_resize(ctx) {
                resize_edges(ui, rect);
            }
        });
        output.textures_delta.clear();
        output.viewport_output.get_mut(&egui::ViewportId::ROOT).map(|root| std::mem::take(&mut root.commands)).unwrap_or_default()
    }

    fn primary(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE }
    }

    #[test]
    fn the_empty_title_row_drags_and_double_clicks() {
        let pos = pos2(120.0, 20.0);
        let moved = pos + egui::vec2(20.0, 0.0);
        let ctx = egui::Context::default();
        input_frame(&ctx, 0.0, vec![egui::Event::PointerMoved(pos)], false);
        assert!(input_frame(&ctx, 0.1, vec![primary(pos, true)], false).is_empty());
        assert!(input_frame(&ctx, 0.2, vec![egui::Event::PointerMoved(moved)], false).contains(&ViewportCommand::StartDrag));

        for maximized in [false, true] {
            let ctx = egui::Context::default();
            input_frame(&ctx, 0.0, vec![egui::Event::PointerMoved(pos)], maximized);
            for time in [0.1, 0.2] {
                assert!(input_frame(&ctx, time, vec![primary(pos, true)], maximized).is_empty());
                let commands = input_frame(&ctx, time + 0.01, vec![primary(pos, false)], maximized);
                assert_eq!(commands.contains(&ViewportCommand::Maximized(!maximized)), time == 0.2);
            }
        }
    }

    #[test]
    fn edges_request_resize_on_press_only_when_restored() {
        let pos = pos2(1.0, 200.0);
        for maximized in [false, true] {
            let ctx = egui::Context::default();
            input_frame(&ctx, 0.0, vec![egui::Event::PointerMoved(pos)], maximized);
            let commands = input_frame(&ctx, 0.1, vec![primary(pos, true)], maximized);
            assert_eq!(commands.contains(&ViewportCommand::BeginResize(ResizeDirection::West)), !maximized);
        }
    }

    #[test]
    fn resize_bands_cover_each_side_and_corner_without_overlapping_the_content() {
        use ResizeDirection as D;
        let full = Rect::from_min_max(pos2(20.0, 30.0), pos2(220.0, 130.0));
        let bands = resize_bands(full);
        let interior = full.shrink(f32::from(RESIZE_INSET));
        for (index, (_, _, band)) in bands.iter().enumerate() {
            assert!(band.is_positive());
            assert!(full.contains_rect(*band));
            assert!(!band.intersect(interior).is_positive());
            for (_, _, other) in bands.iter().skip(index + 1) {
                assert!(!band.intersect(*other).is_positive());
            }
        }
        for (position, expected) in [
            (pos2(120.0, 31.0), D::North),
            (pos2(120.0, 129.0), D::South),
            (pos2(21.0, 80.0), D::West),
            (pos2(219.0, 80.0), D::East),
            (pos2(21.0, 31.0), D::NorthWest),
            (pos2(219.0, 31.0), D::NorthEast),
            (pos2(21.0, 129.0), D::SouthWest),
            (pos2(219.0, 129.0), D::SouthEast),
        ] {
            assert!(bands.iter().any(|(direction, _, band)| *direction == expected && band.contains(position)));
        }
    }
}
