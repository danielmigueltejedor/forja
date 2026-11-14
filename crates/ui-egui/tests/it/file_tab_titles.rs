//! A long drawing name is cut short with "…" inside its document tab in the title bar instead
//! of running under the next tab and the close button (#513); the unsaved-changes mark stays
//! visible after it.

use egui::{Pos2, Rect, Shape, vec2};
use egui_kittest::Harness;
use forja_engine::Session;
use forja_ui_egui::{CadApp, Services};

/// Painted texts (the full source text, the painted bounds, whether it was cut short) and the
/// centres of painted dots.
fn texts(shape: &Shape, out: &mut Vec<(String, Rect, bool)>, dots: &mut Vec<Pos2>) {
    match shape {
        Shape::Text(t) => out.push((t.galley.text().to_string(), t.visual_bounding_rect(), t.galley.elided)),
        Shape::Circle(c) => dots.push(c.center),
        Shape::Vec(v) => v.iter().for_each(|s| texts(s, out, dots)),
        _ => {}
    }
}

#[test]
fn long_names_are_cut_short_inside_the_tab() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", serde_json::json!({})).unwrap();
    app.run("circle", serde_json::json!({"center": [0, 0], "radius": 1})).unwrap();
    app.session.state_mut().unwrap().title = "LICENCIA COFRE DE PEROTE REPOSICION FINAL.dwg".into();
    app.ui.start_tab = false;
    let mut h = Harness::builder().with_size(vec2(1200.0, 760.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    let mut found = Vec::new();
    let mut dots = Vec::new();
    for s in &h.output().shapes {
        texts(&s.shape, &mut found, &mut dots);
    }
    // The window title shows the whole name too; the tab's is the lower one.
    let (_, rect, cut) =
        found.iter().filter(|(t, ..)| t.starts_with("LICENCIA")).max_by(|a, b| a.1.top().total_cmp(&b.1.top())).cloned().expect("the tab's name");
    assert!(cut, "the name ends in an ellipsis");
    // Tabs are at most 240 points wide: 12 of padding, then the name, then 30 for the close
    // button, which shows the unsaved mark as a dot.
    assert!(rect.width() <= 240.0 - 12.0 - 30.0, "{} points wide", rect.width());
    let mark = dots.iter().find(|c| c.x > rect.right() && c.x - rect.right() < 30.0 && (c.y - rect.center().y).abs() < 4.0);
    assert!(mark.is_some(), "the unsaved mark follows the shortened name");
}
