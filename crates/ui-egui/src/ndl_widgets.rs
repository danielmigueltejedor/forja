//! egui's own small controls drawn in the Nodalia Design Language: check boxes that fill with
//! the accent, radio buttons with an accent dot, and combo boxes with NDL's chevron instead of a
//! filled triangle.
//!
//! [`NdlUi`] adds `check`, `radio_choice` and `choice` beside egui's `checkbox`, `radio_value` and `radio`, with the
//! same arguments and the same accessibility roles, so a call site moves over by its name.

use egui::{Color32, Rect, Response, Sense, Stroke, TextStyle, Ui, Widget, WidgetInfo, WidgetText, pos2, vec2};

use crate::theme::{ndl_color, ndl_palette};

/// The side of a check box and the diameter of a radio button.
const BOX: f32 = 16.0;
/// The space between the box and its label.
const GAP: f32 = 6.0;

pub trait NdlUi {
    /// An NDL check box bound to `checked` (egui's `checkbox`).
    fn check(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> Response;
    /// An NDL radio button that sets `current` to `alternative` (egui's `radio_value`).
    fn radio_choice<V: PartialEq>(&mut self, current: &mut V, alternative: V, text: impl Into<WidgetText>) -> Response;
    /// An NDL radio button showing `selected`; the caller acts on the click (egui's `radio`).
    fn choice(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response;
}

impl NdlUi for Ui {
    fn check(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> Response {
        Checkbox { checked, text: text.into() }.ui(self)
    }

    fn radio_choice<V: PartialEq>(&mut self, current: &mut V, alternative: V, text: impl Into<WidgetText>) -> Response {
        let selected = *current == alternative;
        let mut response = toggle(self, Kind::Radio, selected, text.into());
        if response.clicked() && !selected {
            *current = alternative;
            response.mark_changed();
        }
        response
    }

    fn choice(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response {
        toggle(self, Kind::Radio, selected, text.into())
    }
}

struct Checkbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut response = toggle(ui, Kind::Check, *self.checked, self.text);
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        response
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Check,
    Radio,
}

/// Lay out and paint a box (or a disc) with its label; the caller applies the click.
fn toggle(ui: &mut Ui, kind: Kind, on: bool, text: WidgetText) -> Response {
    let wrap = (ui.available_width() - BOX - GAP).max(0.0);
    let galley = text.into_galley(ui, None, wrap, TextStyle::Body);
    let has_text = !galley.is_empty();
    let width = BOX + if has_text { GAP + galley.size().x } else { 0.0 };
    let height = galley.size().y.max(BOX).max(ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let role = if kind == Kind::Check { egui::Role::CheckBox } else { egui::Role::RadioButton };
    response.widget_info(|| WidgetInfo::selected(role, ui.is_enabled(), on, galley.text()));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    // Pressed state of the checked value: what the box will look like once released.
    let p = ndl_palette();
    let enabled = ui.is_enabled();
    let hovered = response.hovered() && enabled;
    let accent = ndl_color(if hovered { p.accent_hover } else { p.accent_default });
    let field = ndl_color(if hovered { p.control_bg_hover } else { p.control_field_bg });
    let border = ndl_color(if hovered { p.border_strong } else { p.border_default });
    let mark = ndl_color(p.text_on_accent);
    let box_rect = Rect::from_center_size(pos2(rect.left() + BOX / 2.0, rect.center().y), vec2(BOX, BOX));
    let painter = ui.painter();
    let fade = |c: Color32| if enabled { c } else { c.gamma_multiply(0.45) };
    match kind {
        Kind::Check => {
            if on {
                painter.rect_filled(box_rect, 5.0, fade(accent));
                let s = Stroke::new(2.0, fade(mark));
                let c = box_rect.center();
                painter.line(vec![c + vec2(-3.75, 0.25), c + vec2(-1.0, 3.0), c + vec2(4.0, -2.75)], s);
            } else {
                painter.rect(box_rect, 5.0, fade(field), Stroke::new(1.5, fade(border)), egui::StrokeKind::Inside);
            }
        }
        Kind::Radio => {
            let r = BOX / 2.0;
            if on {
                painter.circle_filled(box_rect.center(), r, fade(accent));
                painter.circle_filled(box_rect.center(), 3.25, fade(mark));
            } else {
                painter.circle(box_rect.center(), r - 0.75, fade(field), Stroke::new(1.5, fade(border)));
            }
        }
    }
    if has_text {
        let color = if enabled { ui.visuals().text_color() } else { ui.visuals().weak_text_color() };
        let top = rect.center().y - galley.size().y / 2.0;
        ui.painter().galley(pos2(box_rect.right() + GAP, top), galley, color);
    }
    response
}

/// A combo box with NDL's chevron (craft_ui's combo with its triangle replaced).
pub fn combo(id_salt: impl egui::AsIdSalt) -> craft_ui::ComboBox<'static> {
    craft_ui::ComboBox::from_id_salt(id_salt).icon(chevron_icon)
}

/// NDL's chevron, pointing down (up while the menu is open), in the widget's text colour.
fn chevron_icon(ui: &Ui, rect: Rect, visuals: &egui::style::WidgetVisuals, open: bool) {
    let c = rect.center();
    let (w, h) = (4.0, 2.25);
    let dir = if open { -1.0 } else { 1.0 };
    let s = Stroke::new(1.75, visuals.fg_stroke.color);
    ui.painter().line(vec![c + vec2(-w, -h * dir), c + vec2(0.0, h * dir), c + vec2(w, -h * dir)], s);
}

#[cfg(test)]
mod tests {
    use super::NdlUi;

    fn click_at(ctx: &egui::Context, pos: egui::Pos2, frame: impl Fn(&mut egui::Ui)) {
        for events in [vec![egui::Event::PointerMoved(pos)], press_release(pos), vec![]] {
            let input = egui::RawInput { events, ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| frame(ui));
            out.textures_delta.clear();
        }
    }

    fn press_release(pos: egui::Pos2) -> Vec<egui::Event> {
        let b = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
        vec![b(true), b(false)]
    }

    #[test]
    fn a_click_toggles_the_check_box() {
        let ctx = egui::Context::default();
        let checked = std::cell::Cell::new(false);
        let rect = std::cell::Cell::new(egui::Rect::NOTHING);
        let draw = |ui: &mut egui::Ui| {
            let mut c = checked.get();
            rect.set(ui.check(&mut c, "Snap").rect);
            checked.set(c);
        };
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| draw(ui));
        out.textures_delta.clear();
        click_at(&ctx, rect.get().center(), draw);
        assert!(checked.get());
    }

    #[test]
    fn a_click_picks_the_radio_choice() {
        let ctx = egui::Context::default();
        let value = std::cell::Cell::new(1);
        let rect = std::cell::Cell::new(egui::Rect::NOTHING);
        let draw = |ui: &mut egui::Ui| {
            let mut v = value.get();
            ui.radio_choice(&mut v, 1, "One");
            rect.set(ui.radio_choice(&mut v, 2, "Two").rect);
            value.set(v);
        };
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| draw(ui));
        out.textures_delta.clear();
        click_at(&ctx, rect.get().center(), draw);
        assert_eq!(value.get(), 2);
    }
}
