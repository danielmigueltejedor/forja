//! Forja's floating windows (dialogs, managers, About, Options) in the Nodalia Design Language:
//! a rounded island with NDL's overlay surface and shadow, a bold title and NDL's round close
//! control instead of egui's title bar with its collapse triangle and thin cross.
//!
//! [`Window`] takes the subset of [`egui::Window`]'s builder the dialogs use, so a dialog moves
//! over by changing its constructor. Without egui's title bar the window is dragged by its
//! background (widgets inside still take their own drags first).

use egui::{Align2, Context, InnerResponse, Pos2, Sense, Ui, Vec2, WidgetText, vec2};

use crate::icons::Icon;
use crate::theme::Tokens;

/// The window corner radius: NDL's panel radius, the same as the docked islands.
pub const RADIUS: f32 = nodalia_theme::component::PANEL_RADIUS;
/// The space between the window edge and its content.
const MARGIN: f32 = 14.0;
/// NDL's round window control.
pub const CONTROL: f32 = 28.0;

/// A floating window in NDL's style. See the module documentation.
#[must_use = "call .show(ctx, …) to draw the window"]
pub struct Window<'open> {
    inner: egui::Window<'open>,
    title: WidgetText,
    open: Option<&'open mut bool>,
    header: bool,
    /// Whether the call site placed the window; an unplaced one opens centred, as NDL's dialogs do.
    placed: bool,
    /// The size the call site expects, to centre an unplaced window on its first frame.
    size: Vec2,
}

impl<'open> Window<'open> {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        let title = title.into();
        Self { inner: egui::Window::new(title.clone()), title, open: None, header: true, placed: false, size: vec2(420.0, 320.0) }
    }

    pub fn id(mut self, id: egui::Id) -> Self {
        self.inner = self.inner.id(id);
        self
    }

    /// Show the round close control; it sets `open` to false.
    pub fn open(mut self, open: &'open mut bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_size(mut self, size: impl Into<Vec2>) -> Self {
        let size = size.into();
        self.size = size;
        self.inner = self.inner.default_size(size);
        self
    }

    pub fn default_width(mut self, w: f32) -> Self {
        self.size.x = w;
        self.inner = self.inner.default_width(w);
        self
    }

    pub fn default_height(mut self, h: f32) -> Self {
        self.size.y = h;
        self.inner = self.inner.default_height(h);
        self
    }

    pub fn min_width(mut self, w: f32) -> Self {
        self.inner = self.inner.min_width(w);
        self
    }

    pub fn max_height(mut self, h: f32) -> Self {
        self.inner = self.inner.max_height(h);
        self
    }

    pub fn default_pos(mut self, pos: impl Into<Pos2>) -> Self {
        self.placed = true;
        self.inner = self.inner.default_pos(pos);
        self
    }

    pub fn anchor(mut self, align: Align2, offset: impl Into<Vec2>) -> Self {
        self.placed = true;
        self.inner = self.inner.anchor(align, offset);
        self
    }

    pub fn pivot(mut self, pivot: Align2) -> Self {
        self.placed = true;
        self.inner = self.inner.pivot(pivot);
        self
    }

    pub fn resizable(mut self, resizable: impl Into<egui::Vec2b>) -> Self {
        self.inner = self.inner.resizable(resizable);
        self
    }

    /// NDL windows never collapse; kept so call sites read like egui's builder.
    pub fn collapsible(self, _collapsible: bool) -> Self {
        self
    }

    /// False drops the title and close control as well (a bare island).
    pub fn title_bar(mut self, title_bar: bool) -> Self {
        self.header = title_bar;
        self
    }

    pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<Option<R>>> {
        let Self { mut inner, title, open, header, placed, size } = self;
        if !placed {
            // Centred by its expected size rather than pivoted on its centre: a centre pivot would
            // move the window every time its content grows or shrinks.
            let screen = ctx.content_rect();
            let size = size.min(screen.size());
            inner = inner.default_pos(screen.center() - size / 2.0);
        }
        if open.as_deref() == Some(&false) {
            return None;
        }
        let closable = open.is_some();
        let mut close = false;
        let shown = inner.title_bar(false).frame(frame(ctx)).show(ctx, |ui| {
            if header {
                close = title_row(ui, title, closable);
            }
            add_contents(ui)
        });
        if close && let Some(open) = open {
            *open = false;
        }
        shown
    }
}

/// The island every NDL window is drawn on.
pub fn frame(ctx: &Context) -> egui::Frame {
    let style = ctx.style_of(ctx.theme());
    let v = &style.visuals;
    egui::Frame::window(&style).corner_radius(RADIUS).inner_margin(MARGIN).fill(v.window_fill).stroke(v.window_stroke)
}

/// A surface one step above the window's: NDL's grouped cards, pills and round controls.
/// `level` 1 and 2 are its hovered and pressed states.
pub fn raised(ui: &Ui, level: u8) -> egui::Color32 {
    let v = ui.visuals();
    let step = 0.07 + 0.04 * f32::from(level.min(2));
    let toward = if v.dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK };
    v.window_fill.lerp_to_gamma(toward, step)
}

/// The title and the close control on one row. True when the close control was clicked.
fn title_row(ui: &mut Ui, title: WidgetText, closable: bool) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        ui.set_min_height(CONTROL);
        let t = Tokens::get();
        ui.label(title.strong().size(15.0).color(t.text));
        if closable {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                close = close_control(ui).clicked();
            });
        }
    });
    ui.add_space(6.0);
    close
}

/// NDL's round close control: a 28 px disc that lights up on hover, with NDL's bold cross.
pub fn close_control(ui: &mut Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(CONTROL, CONTROL), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Close"));
    let t = Tokens::get();
    let fill = raised(ui, if response.is_pointer_button_down_on() { 2 } else { u8::from(response.hovered()) });
    ui.painter().circle_filled(rect.center(), CONTROL / 2.0, fill);
    let glyph = egui::Rect::from_center_size(rect.center(), vec2(14.0, 14.0));
    if let Some(body) = crate::ndl_icons::body(Icon::Close) {
        crate::ndl_icons::paint(ui.painter(), glyph, body, t.text);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::Window;

    /// Run two frames of a window and report whether it drew anything.
    fn shows(open: &mut bool) -> bool {
        let ctx = egui::Context::default();
        let mut drawn = false;
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                let ctx = ui.ctx().clone();
                drawn = Window::new("Test").open(&mut *open).show(&ctx, |ui| ui.label("body")).is_some();
            });
            out.textures_delta.clear();
        }
        drawn
    }

    #[test]
    fn a_closed_window_shows_nothing_and_stays_closed() {
        let mut open = false;
        assert!(!shows(&mut open));
        assert!(!open);
    }

    #[test]
    fn an_open_window_shows_and_stays_open_until_closed() {
        let mut open = true;
        assert!(shows(&mut open));
        assert!(open);
    }
}
