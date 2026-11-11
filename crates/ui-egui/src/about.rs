//! The About window, laid out like every Nodalia app's About: the app icon, its name, who makes
//! it, the version as a pill and one line on what it is, then a card of rows (website,
//! contributors, models, licence). Contributors and Models open as pages of the same window.

use egui::{Align, Color32, Layout, RichText, Sense, TextureHandle, Ui, vec2};

use crate::icons::Icon;
use crate::ndl_window;
use crate::theme::Tokens;

const WEBSITE: &str = "https://github.com/danielmigueltejedor/forja";
const ICON_PNG: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/128x128/apps/ai.storyteller.forja.png");
/// The width of the About page: NDL's dialog width.
const WIDTH: f32 = 380.0;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    About,
    Contributors,
    Models,
}

pub fn show(ctx: &egui::Context, open: &mut bool) {
    let page_id = egui::Id::unique("about_page");
    let mut page = ctx.data_mut(|d| d.get_temp::<Page>(page_id)).unwrap_or_default();
    let title = match page {
        Page::About => String::new(),
        Page::Contributors => crate::tl!("Contributors").to_string(),
        Page::Models => crate::tl!("Models").to_string(),
    };
    ndl_window::Window::new(title)
        .id(egui::Id::unique("About Forja"))
        .open(open)
        .resizable(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .show(ctx, |ui| match page {
            Page::About => {
                ui.set_width(WIDTH);
                about_page(ui, &mut page);
            }
            Page::Contributors | Page::Models => {
                if back_row(ui) {
                    page = Page::About;
                }
                ui.add_space(6.0);
                if page == Page::Contributors {
                    crate::credits::contributors_ui(ui);
                } else {
                    crate::credits::models_ui(ui);
                }
            }
        });
    if !*open {
        page = Page::About;
    }
    ctx.data_mut(|d| d.insert_temp(page_id, page));
}

fn about_page(ui: &mut Ui, page: &mut Page) {
    let t = Tokens::get();
    ui.vertical_centered(|ui| {
        if let Some(icon) = icon(ui.ctx()) {
            ui.add(egui::Image::new(&icon).fit_to_exact_size(vec2(112.0, 112.0)));
        }
        ui.add_space(4.0);
        ui.label(RichText::new("Forja").size(26.0).strong().color(t.text));
        ui.label(RichText::new("Daniel Miguel Tejedor").color(t.text_dim));
        ui.add_space(4.0);
        pill(ui, env!("CARGO_PKG_VERSION"));
        ui.add_space(8.0);
        ui.label(
            RichText::new(crate::tl!("Computer-aided design and drafting: an open-source, clean-room CAD application written in pure Rust."))
                .color(t.text),
        );
    });
    ui.add_space(14.0);
    card(ui, |ui| {
        if row(ui, Some(Icon::Share), crate::tl!("Website"), None, true) {
            ui.ctx().open_url(egui::OpenUrl::new_tab(WEBSITE));
        }
        separator(ui);
        if row(ui, None, crate::tl!("Contributors"), None, true) {
            *page = Page::Contributors;
        }
        separator(ui);
        if row(ui, None, crate::tl!("Models"), None, true) {
            *page = Page::Models;
        }
        separator(ui);
        row(ui, None, crate::tl!("Licence"), Some("MIT OR Apache-2.0"), false);
    });
    ui.add_space(10.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(crate::tl!("Not affiliated with Autodesk, Inc.")).small().color(t.text_faint));
    });
}

/// The app icon as a texture, decoded once.
fn icon(ctx: &egui::Context) -> Option<TextureHandle> {
    let id = egui::Id::unique("about_icon_texture");
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let rgba = image::load_from_memory(ICON_PNG).ok()?.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let texture = ctx.load_texture("about_icon", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    Some(texture)
}

/// A small rounded label: NDL's pill, as wide as its text.
fn pill(ui: &mut Ui, text: &str) {
    let p = crate::theme::ndl_palette();
    let font = egui::TextStyle::Small.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, crate::theme::ndl_color(p.accent_text));
    let size = galley.size() + vec2(20.0, 6.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, size.y / 2.0, crate::theme::ndl_color(p.accent_subtle));
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::PLACEHOLDER);
}

/// NDL's grouped card: rows on a raised surface with rounded corners.
fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new().fill(ndl_window::raised(ui, 0)).corner_radius(12.0).inner_margin(egui::Margin::symmetric(4, 4)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        add(ui);
    });
}

fn separator(ui: &mut Ui) {
    let t = Tokens::get();
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().hline(rect.x_range().shrink(10.0), rect.center().y, egui::Stroke::new(1.0, t.border));
}

/// One row of a card. True when an activatable row was clicked.
fn row(ui: &mut Ui, icon: Option<Icon>, title: &str, value: Option<&str>, activatable: bool) -> bool {
    let t = Tokens::get();
    let w = ui.available_width();
    let sense = if activatable { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(vec2(w, 40.0), sense);
    if activatable && response.hovered() {
        ui.painter().rect_filled(rect, 9.0, ndl_window::raised(ui, 2));
    }
    let inner = rect.shrink2(vec2(12.0, 0.0));
    let mut x = inner.left();
    if let Some(icon) = icon {
        let r = egui::Rect::from_center_size(egui::pos2(x + 9.0, inner.center().y), vec2(18.0, 18.0));
        crate::ndl_icons::paint_ink(ui.painter(), r, icon, t.text_dim);
        x += 28.0;
    }
    let font = egui::TextStyle::Body.resolve(ui.style());
    ui.painter().text(egui::pos2(x, inner.center().y), egui::Align2::LEFT_CENTER, title, font.clone(), t.text);
    if let Some(value) = value {
        ui.painter().text(inner.right_center(), egui::Align2::RIGHT_CENTER, value, font, t.text_dim);
    } else if activatable {
        chevron(ui, inner.right_center(), t.text_dim);
    }
    let response = response.on_hover_cursor(if activatable { egui::CursorIcon::PointingHand } else { egui::CursorIcon::Default });
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title));
    activatable && response.clicked()
}

/// A ›, drawn so it matches NDL's stroke rather than a font's glyph.
fn chevron(ui: &Ui, right_center: egui::Pos2, color: Color32) {
    let c = right_center - vec2(5.0, 0.0);
    let s = egui::Stroke::new(1.75, color);
    ui.painter().line_segment([c + vec2(-3.0, -5.0), c + vec2(2.0, 0.0)], s);
    ui.painter().line_segment([c + vec2(2.0, 0.0), c + vec2(-3.0, 5.0)], s);
}

/// The way back to the About page from Contributors or Models. True when clicked.
fn back_row(ui: &mut Ui) -> bool {
    let t = Tokens::get();
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        let text = format!("‹  {}", crate::tl!("About"));
        ui.add(egui::Button::new(RichText::new(text).color(t.text)).fill(ndl_window::raised(ui, 0)).corner_radius(f32::from(u8::MAX))).clicked()
    })
    .inner
}
