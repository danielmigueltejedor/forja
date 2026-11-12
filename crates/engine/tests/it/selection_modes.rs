//! Selection modes typed at "Select objects": Window, Crossing, Fence, WPolygon, CPolygon.

use forja_engine::Session;
use forja_engine::doc::EntityKind;

/// Three lines on the X axis: A 0–10, B 12–20, C 30–40.
fn drawing() -> Session {
    let mut s = Session::new();
    for l in ["LINE 0,0 10,0", "LINE 12,0 20,0", "LINE 30,0 40,0"] {
        s.cmdline(l).unwrap();
        s.cmdline("").unwrap();
    }
    s
}

/// The start X of the lines left in the drawing.
fn left(s: &Session) -> Vec<f64> {
    let mut v: Vec<f64> = s.doc().unwrap().model.iter().filter_map(|e| if let EntityKind::Line(l) = &e.kind { Some(l.a.x) } else { None }).collect();
    v.sort_by(f64::total_cmp);
    v
}

fn erase(input: &str) -> Vec<f64> {
    let mut s = drawing();
    s.cmdline(input).unwrap();
    assert!(s.running.is_some(), "{input}: still selecting");
    s.cmdline("").unwrap();
    assert!(s.running.is_none(), "{input}: Enter ends the selection");
    left(&s)
}

#[test]
fn crossing_and_window_ignore_the_drag_direction() {
    // Left to right would be a window; C makes it a crossing: A (touching) and B (inside).
    assert_eq!(erase("ERASE C 5,-1 25,1"), vec![30.0]);
    // Right to left would be a crossing; W makes it a window: only B.
    assert_eq!(erase("ERASE W 25,1 5,-1"), vec![0.0, 30.0]);
    // Full words work too.
    assert_eq!(erase("ERASE crossing 25,1 5,-1"), vec![30.0]);
}

#[test]
fn fence_selects_what_it_crosses() {
    let mut s = drawing();
    // Enter ends the fence, a second Enter ends "Select objects".
    s.cmdline("ERASE F 6,-1 6,1 35,1 35,-1").unwrap();
    s.cmdline("").unwrap();
    assert!(s.running.is_some());
    s.cmdline("").unwrap();
    assert!(s.running.is_none());
    // Crosses A at x=6 and C at x=35, but not B.
    assert_eq!(left(&s), vec![12.0]);
    // Undo drops the last fence point.
    let mut s = drawing();
    s.cmdline("ERASE F 6,-1 6,1 35,1 U").unwrap();
    s.cmdline("").unwrap();
    s.cmdline("").unwrap();
    assert_eq!(left(&s), vec![12.0, 30.0]);
}

#[test]
fn window_and_crossing_polygons() {
    let mut s = drawing();
    s.cmdline("ERASE WP 5,-1 25,-1 25,1 5,1").unwrap();
    s.cmdline("").unwrap();
    s.cmdline("").unwrap();
    assert_eq!(left(&s), vec![0.0, 30.0]);
    let mut s = drawing();
    s.cmdline("ERASE CP 5,-1 25,-1 25,1 5,1").unwrap();
    s.cmdline("").unwrap();
    s.cmdline("").unwrap();
    assert_eq!(left(&s), vec![30.0]);
}

#[test]
fn escape_during_a_mode_cancels_the_command() {
    let mut s = drawing();
    s.cmdline("ERASE C").unwrap();
    assert_eq!(s.current_prompt().map(|p| p.message), Some("Specify first corner".to_string()));
    s.cmdline("5,-1").unwrap();
    assert_eq!(s.current_prompt().map(|p| p.message), Some("Specify opposite corner".to_string()));
    s.cancel();
    assert!(s.running.is_none());
    assert_eq!(left(&s), vec![0.0, 12.0, 30.0]);
    // The next command starts at a plain "Select objects".
    s.cmdline("ERASE 15,0").unwrap();
    s.cmdline("").unwrap();
    assert_eq!(left(&s), vec![0.0, 30.0]);
}

/// The lasso a press-and-drag draws: a loop around A and the left half of B.
fn lasso(rightwards: bool) -> Vec<forja_engine::geom::Vec2> {
    use forja_engine::geom::Vec2;
    let mut pts = vec![
        Vec2::new(-2.0, -3.0),
        Vec2::new(4.0, -4.0),
        Vec2::new(11.0, -3.0),
        Vec2::new(16.0, -1.0),
        Vec2::new(15.0, 3.0),
        Vec2::new(5.0, 4.0),
        Vec2::new(-3.0, 2.0),
    ];
    if !rightwards {
        // The same loop, set off leftwards from its right-hand end.
        pts.rotate_left(3);
        pts.reverse();
    }
    pts
}

fn selected_starts(s: &Session) -> Vec<f64> {
    let d = s.doc().unwrap();
    let mut v: Vec<f64> =
        s.selection().iter().filter_map(|h| if let EntityKind::Line(l) = &d.entity(*h)?.kind { Some(l.a.x) } else { None }).collect();
    v.sort_by(f64::total_cmp);
    v
}

#[test]
fn a_rightwards_lasso_selects_only_what_it_encloses() {
    let mut s = drawing();
    let pts = lasso(true);
    assert!(!s.lasso_is_crossing(&pts));
    s.lasso(&pts, false).unwrap();
    assert_eq!(selected_starts(&s), [0.0], "B is only half inside");
}

#[test]
fn a_leftwards_lasso_selects_what_it_touches() {
    let mut s = drawing();
    let pts = lasso(false);
    assert!(s.lasso_is_crossing(&pts));
    s.lasso(&pts, false).unwrap();
    assert_eq!(selected_starts(&s), [0.0, 12.0]);
}

#[test]
fn a_lasso_at_select_objects_feeds_the_command() {
    let mut s = drawing();
    s.cmdline("ERASE").unwrap();
    s.lasso(&lasso(false), false).unwrap();
    s.cmdline("").unwrap();
    assert_eq!(left(&s), [30.0]);
}

#[test]
fn degenerate_or_hostile_lassos_select_nothing() {
    use forja_engine::geom::Vec2;
    let mut s = drawing();
    s.lasso(&[Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)], false).unwrap();
    s.lasso(&[Vec2::new(0.0, 0.0), Vec2::new(f64::NAN, 1.0), Vec2::new(10.0, 0.0)], false).unwrap();
    assert!(s.selection().is_empty());
    // A huge outline is thinned, not refused.
    let n = 100_000;
    let big: Vec<Vec2> =
        (0..n).map(|i| std::f64::consts::TAU * f64::from(i) / f64::from(n)).map(|a| Vec2::new(5.0 - 18.0 * a.cos(), 18.0 * a.sin())).collect();
    s.lasso(&big, false).unwrap();
    assert_eq!(selected_starts(&s), [0.0, 12.0], "a rightwards circle around A and B, clear of C");
}

#[test]
fn shift_lasso_removes_from_the_selection() {
    let mut s = drawing();
    s.lasso(&lasso(false), false).unwrap();
    s.lasso(&lasso(true), true).unwrap();
    assert_eq!(selected_starts(&s), [12.0]);
}
