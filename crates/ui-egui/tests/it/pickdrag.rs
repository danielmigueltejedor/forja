//! OPTIONS checkbox and real pointer gestures share the saved PICKDRAG preference: with it on, a
//! press-and-drag draws a lasso; click, move, click makes the rectangular window.
use cadcraft_engine::Session;
use cadcraft_engine::geom::Vec2;
use cadcraft_ui_egui::{CadApp, Services};
use egui::{Event, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use serde_json::json;

fn harness() -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::new(), Services::default());
    app.canvas.zoom_pending = false;
    app.session.execute("line", &json!({"points":[[0,0],[10,0]]})).unwrap();
    app.session.execute("view.set", &json!({"center":[5,0],"height":30})).unwrap();
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(3);
    h
}

fn button(h: &mut Harness<'_, CadApp>, p: Pos2, pressed: bool) {
    h.event(Event::PointerMoved(p));
    h.event(Event::PointerButton { pos: p, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
    h.run_steps(1);
}

fn drag(h: &mut Harness<'_, CadApp>, a: Pos2, b: Pos2) {
    button(h, a, true);
    for i in 1..=4 {
        h.event(Event::PointerMoved(a + (b - a) * (i as f32 / 4.0)));
        h.run_steps(1);
    }
    button(h, b, false);
    h.run_steps(2);
}

#[test]
fn selection_tab_checkbox_updates_and_persists_the_preference() {
    let mut h = harness();
    h.state_mut().ui.dialog = Some("options:selection".into());
    h.run_steps(3);
    h.get_by_label("Select using two clicks only").click();
    h.run_steps(2);
    assert!(!h.state().session.settings.pickdrag);
    let saved = h.state().profile_json();
    let mut restored = CadApp::new(Session::new(), Services::default());
    restored.load_profile(&saved);
    assert!(!restored.session.settings.pickdrag);
    if let Some(dir) = std::env::var_os("CADCRAFT_SELECTION_SHOTS") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        h.render().unwrap().save(dir.join("selection-two-clicks.png")).unwrap();
    }
    h.get_by_label("Select using two clicks only").click();
    h.run_steps(2);
    assert!(h.state().session.settings.pickdrag);
}

#[test]
fn drag_is_disabled_but_two_clicks_still_complete_the_window() {
    for prompting in [false, true] {
        let mut h = harness();
        h.state_mut().run("options", json!({"PICKDRAG":0})).unwrap();
        if prompting {
            h.state_mut().session.start("select").unwrap();
        }
        h.run_steps(2);
        let xf = h.state().canvas.xf.unwrap();
        let a = xf.to_screen(Vec2::new(-2.0, -2.0));
        let b = xf.to_screen(Vec2::new(12.0, 2.0));
        drag(&mut h, a, b);
        assert!(h.state().session.selection().is_empty(), "drag must not complete selection");
        h.state_mut().session.cancel();
        h.state_mut().session.set_selection(Vec::new());
        if prompting {
            h.state_mut().session.start("select").unwrap();
        }
        h.run_steps(2);
        button(&mut h, a, true);
        button(&mut h, a, false);
        assert!(h.state().session.pending_window.is_some());
        // A held-button movement after the first click must not consume the second corner.
        drag(&mut h, a, b);
        assert!(h.state().session.pending_window.is_some());
        assert!(h.state().session.selection().is_empty());
        h.run_steps(30); // Separate deliberate clicks from egui's double-click interval.
        button(&mut h, b, true);
        button(&mut h, b, false);
        assert!(h.state().session.pending_window.is_none());
        assert_eq!(h.state().session.selection().len(), 1);
    }
}

/// Press at the first point, move through the rest, let go at the last.
fn drag_path(h: &mut Harness<'_, CadApp>, pts: &[Vec2]) {
    let xf = h.state().canvas.xf.unwrap();
    let screen: Vec<Pos2> = pts.iter().map(|p| xf.to_screen(*p)).collect();
    button(h, screen[0], true);
    for p in &screen[1..] {
        h.event(Event::PointerMoved(*p));
        h.run_steps(1);
    }
    button(h, *screen.last().unwrap(), false);
    h.run_steps(2);
}

/// A loop around the line 0–10, set off rightwards (a window lasso).
fn around_the_line() -> Vec<Vec2> {
    vec![Vec2::new(-2.0, -2.0), Vec2::new(5.0, -3.0), Vec2::new(12.0, -2.0), Vec2::new(13.0, 2.0), Vec2::new(5.0, 3.0), Vec2::new(-3.0, 2.0)]
}

#[test]
fn a_drag_draws_a_lasso_not_a_rectangle() {
    let mut h = harness();
    let xf = h.state().canvas.xf.unwrap();
    let pts = around_the_line();
    let screen: Vec<Pos2> = pts.iter().map(|p| xf.to_screen(*p)).collect();
    button(&mut h, screen[0], true);
    for p in &screen[1..4] {
        h.event(Event::PointerMoved(*p));
        h.run_steps(1);
    }
    let lasso = h.state().canvas.lasso.clone().expect("a lasso while the button is held");
    assert!(lasso.len() >= 3, "the lasso follows the pointer: {lasso:?}");
    assert!(h.state().session.pending_window.is_none(), "no rectangular window while dragging");
    for p in &screen[4..] {
        h.event(Event::PointerMoved(*p));
        h.run_steps(1);
    }
    button(&mut h, *screen.last().unwrap(), false);
    h.run_steps(2);
    assert!(h.state().canvas.lasso.is_none());
    assert_eq!(h.state().session.selection().len(), 1, "the window lasso encloses the line");
}

#[test]
fn a_window_lasso_needs_the_whole_object_and_a_crossing_one_a_touch() {
    // Around the line's left half only.
    let half = |rightwards: bool| {
        if rightwards {
            vec![Vec2::new(-2.0, -2.0), Vec2::new(3.0, -3.0), Vec2::new(6.0, -2.0), Vec2::new(6.0, 2.0), Vec2::new(2.0, 3.0), Vec2::new(-3.0, 2.0)]
        } else {
            vec![Vec2::new(6.0, 2.0), Vec2::new(2.0, 3.0), Vec2::new(-3.0, 2.0), Vec2::new(-2.0, -2.0), Vec2::new(3.0, -3.0), Vec2::new(6.0, -2.0)]
        }
    };
    let mut h = harness();
    drag_path(&mut h, &half(true));
    assert!(h.state().session.selection().is_empty(), "a window lasso must enclose the whole line");
    h.run_steps(30);
    drag_path(&mut h, &half(false));
    assert_eq!(h.state().session.selection().len(), 1, "a crossing lasso selects what it touches");
}

#[test]
fn a_straight_drag_selects_nothing_and_leaves_no_window() {
    let mut h = harness();
    let xf = h.state().canvas.xf.unwrap();
    drag(&mut h, xf.to_screen(Vec2::new(-2.0, -2.0)), xf.to_screen(Vec2::new(12.0, 2.0)));
    assert!(h.state().session.pending_window.is_none());
    assert!(h.state().session.selection().is_empty());
}

#[test]
fn click_move_click_still_makes_the_rectangular_window() {
    let mut h = harness();
    let xf = h.state().canvas.xf.unwrap();
    let (a, b) = (xf.to_screen(Vec2::new(-2.0, -2.0)), xf.to_screen(Vec2::new(12.0, 2.0)));
    button(&mut h, a, true);
    button(&mut h, a, false);
    assert!(h.state().session.pending_window.is_some());
    h.event(Event::PointerMoved(b));
    h.run_steps(30);
    button(&mut h, b, true);
    button(&mut h, b, false);
    assert!(h.state().session.pending_window.is_none());
    assert_eq!(h.state().session.selection().len(), 1);
}

#[test]
fn a_lasso_at_select_objects_goes_to_the_command() {
    let mut h = harness();
    h.state_mut().session.start("erase").unwrap();
    h.run_steps(2);
    drag_path(&mut h, &around_the_line());
    h.state_mut().session.input(cadcraft_engine::Input::Enter).unwrap();
    h.run_steps(2);
    assert_eq!(h.state().session.doc().unwrap().model.iter().count(), 0, "ERASE took the lassoed line");
}
