//! The generic interface icons (files, edit, view, navigation, chevrons) in the Nodalia Design
//! Language's bold style: NDL's own shapes, a 2.25 stroke on the 24 grid with round caps and
//! joins, and solid where NDL fills them. The shapes are the SVG bodies of NDL's icon set
//! (`packages/icons` in danielmigueltejedor/nodalia-design, MIT), drawn here by a small SVG
//! subset painter (paths with M L H V C A Z, rects with rx, circles). CAD-specific icons stay
//! Forja's own ([`crate::icons`]).

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2};

use crate::icons::Icon;

/// The NDL stroke on the 24 x 24 grid.
pub const BOLD_STROKE: f32 = 2.25;

/// The generic icons NDL draws, with its bold bodies.
const MAPPED: &[(Icon, &[&str])] = &[
    (
        Icon::New,
        &[
            "<path d=\"M13.5 3h-6A1.5 1.5 0 0 0 6 4.5v15A1.5 1.5 0 0 0 7.5 21h9a1.5 1.5 0 0 0 1.5-1.5v-12z\"/>",
            "<path d=\"M13.5 3v4.5H18\"/>",
            "<path d=\"M12 11.5v6M9 14.5h6\"/>",
        ],
    ),
    (
        Icon::Open,
        &[
            "<path d=\"M3.5 18.5v-12A1.5 1.5 0 0 1 5 5h4l2 2.5h8A1.5 1.5 0 0 1 20.5 9v9.5A1.5 1.5 0 0 1 19 20H5a1.5 1.5 0 0 1-1.5-1.5z\"/>",
            "<path d=\"M3.5 11h17\"/>",
        ],
    ),
    (
        Icon::Save,
        &[
            "<path d=\"M5.5 4H16l4 4v10.5a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 18.5v-13A1.5 1.5 0 0 1 5.5 4z\"/>",
            "<path d=\"M8 4v4.5h6.5V4\"/>",
            "<path d=\"M8 20v-6h8v6\"/>",
        ],
    ),
    (Icon::Undo, &["<path d=\"M8 5 3.5 9.5 8 14\"/>", "<path d=\"M3.5 9.5h11a5.25 5.25 0 0 1 0 10.5H11\"/>"]),
    (Icon::Redo, &["<path d=\"M16 5l4.5 4.5L16 14\"/>", "<path d=\"M20.5 9.5h-11a5.25 5.25 0 0 0 0 10.5H13\"/>"]),
    (
        Icon::Plot,
        &[
            "<path d=\"M7 8V4h10v4\"/>",
            "<path d=\"M7 16.5H5A1.5 1.5 0 0 1 3.5 15V9.5A1.5 1.5 0 0 1 5 8h14a1.5 1.5 0 0 1 1.5 1.5V15a1.5 1.5 0 0 1-1.5 1.5h-2\"/>",
            "<rect x=\"7\" y=\"13.5\" width=\"10\" height=\"7\" rx=\"0.5\"/>",
        ],
    ),
    (
        Icon::Import,
        &[
            "<path d=\"M12 3.5v11\"/>",
            "<path d=\"M7.5 10.5 12 15l4.5-4.5\"/>",
            "<path d=\"M4 16v3a1.5 1.5 0 0 0 1.5 1.5h13A1.5 1.5 0 0 0 20 19v-3\"/>",
        ],
    ),
    (Icon::Attach, &["<path d=\"M19.5 11.5l-7.8 7.8a4.5 4.5 0 0 1-6.4-6.4l8.3-8.3a3 3 0 0 1 4.3 4.3l-8.3 8.2a1.5 1.5 0 0 1-2.1-2.1l7.5-7.5\"/>"]),
    (
        Icon::Share,
        &[
            "<circle cx=\"6\" cy=\"12\" r=\"2.5\"/>",
            "<circle cx=\"18\" cy=\"6\" r=\"2.5\"/>",
            "<circle cx=\"18\" cy=\"18\" r=\"2.5\"/>",
            "<path d=\"M8.2 10.9l7.6-3.8M8.2 13.1l7.6 3.8\"/>",
        ],
    ),
    (Icon::ZoomIn, &["<circle cx=\"10.5\" cy=\"10.5\" r=\"6.5\"/>", "<path d=\"M15.5 15.5l5 5\"/>", "<path d=\"M10.5 8v5M8 10.5h5\"/>"]),
    (Icon::ZoomOut, &["<circle cx=\"10.5\" cy=\"10.5\" r=\"6.5\"/>", "<path d=\"M15.5 15.5l5 5\"/>", "<path d=\"M8 10.5h5\"/>"]),
    (
        Icon::Pan,
        &[
            "<path d=\"M12 3v18M3 12h18\"/>",
            "<path d=\"M9.5 5.5 12 3l2.5 2.5M9.5 18.5 12 21l2.5-2.5M5.5 9.5 3 12l2.5 2.5M18.5 9.5 21 12l-2.5 2.5\"/>",
        ],
    ),
    (Icon::Orbit, &["<circle cx=\"12\" cy=\"12\" r=\"2.5\"/>", "<path d=\"M19.9 10.5a8 8 0 1 0-.9 5.4\"/>", "<path d=\"M20.5 5.5v5h-5\"/>"]),
    (
        Icon::ZoomExtents,
        &[
            "<path d=\"M4 8.5v-3A1.5 1.5 0 0 1 5.5 4h3M15.5 4h3A1.5 1.5 0 0 1 20 5.5v3M20 15.5v3a1.5 1.5 0 0 1-1.5 1.5h-3M8.5 20h-3A1.5 1.5 0 0 1 4 18.5v-3\"/>",
            "<rect x=\"8.5\" y=\"8.5\" width=\"7\" height=\"7\" rx=\"1\"/>",
        ],
    ),
    (
        Icon::ZoomFit,
        &[
            "<path d=\"M4 8.5v-3A1.5 1.5 0 0 1 5.5 4h3M15.5 4h3A1.5 1.5 0 0 1 20 5.5v3M20 15.5v3a1.5 1.5 0 0 1-1.5 1.5h-3M8.5 20h-3A1.5 1.5 0 0 1 4 18.5v-3\"/>",
            "<rect x=\"8.5\" y=\"8.5\" width=\"7\" height=\"7\" rx=\"1\"/>",
        ],
    ),
    (
        Icon::Help,
        &[
            "<circle cx=\"12\" cy=\"12\" r=\"8.5\"/>",
            "<path d=\"M9.5 9.5a2.5 2.5 0 1 1 3.6 2.2c-.7.4-1.1.9-1.1 1.8\"/>",
            "<circle cx=\"12\" cy=\"16.75\" r=\"1.15\" fill=\"currentColor\" stroke=\"none\"/>",
        ],
    ),
    (
        Icon::Copy,
        &[
            "<rect x=\"4\" y=\"8\" width=\"12\" height=\"12\" rx=\"1.5\"/>",
            "<path d=\"M8 8V5.5A1.5 1.5 0 0 1 9.5 4h9A1.5 1.5 0 0 1 20 5.5v9a1.5 1.5 0 0 1-1.5 1.5H16\"/>",
        ],
    ),
    (
        Icon::Erase,
        &[
            "<path d=\"M4 7h16\"/>",
            "<path d=\"M9.5 7V5a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v2\"/>",
            "<path d=\"M6.5 7l.8 12.1a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4L17.5 7\"/>",
            "<path d=\"M10 11v5.5M14 11v5.5\"/>",
        ],
    ),
    (Icon::Rotate, &["<path d=\"M19.5 12A7.5 7.5 0 1 1 12 4.5h4\"/>", "<path d=\"M13.5 2 16 4.5 13.5 7\"/>"]),
    (Icon::Lock, &["<rect fill=\"currentColor\" x=\"5\" y=\"11\" width=\"14\" height=\"9.5\" rx=\"1.5\"/>", "<path d=\"M8 11V8a4 4 0 0 1 8 0v3\"/>"]),
    (Icon::Unlock, &["<rect x=\"5\" y=\"11\" width=\"14\" height=\"9.5\" rx=\"1.5\"/>", "<path d=\"M8 11V8a4 4 0 0 1 7.5-2\"/>"]),
    (
        Icon::Sun,
        &[
            "<circle cx=\"12\" cy=\"12\" r=\"4\"/>",
            "<path d=\"M12 2.5V5M12 19v2.5M2.5 12H5M19 12h2.5M5.3 5.3l1.75 1.75M16.95 16.95l1.75 1.75M5.3 18.7l1.75-1.75M16.95 7.05l1.75-1.75\"/>",
        ],
    ),
    (Icon::ChevronDown, &["<path d=\"M6 9.5l6 6 6-6\"/>"]),
    (Icon::ChevronUp, &["<path d=\"M6 14.5l6-6 6 6\"/>"]),
    (Icon::ChevronLeft, &["<path d=\"M14.5 6l-6 6 6 6\"/>"]),
    (Icon::ChevronRight, &["<path d=\"M9.5 6l6 6-6 6\"/>"]),
    (Icon::Plus, &["<path d=\"M12 5v14M5 12h14\"/>"]),
    (Icon::Close, &["<path d=\"M6 6l12 12M18 6 6 18\"/>"]),
    (Icon::Search, &["<circle cx=\"10.5\" cy=\"10.5\" r=\"6.5\"/>", "<path d=\"M15.5 15.5l5 5\"/>"]),
    (Icon::Menu, &["<path d=\"M4 6.5h16M4 12h16M4 17.5h16\"/>"]),
    (Icon::Table, &["<rect x=\"3.5\" y=\"5\" width=\"17\" height=\"14\" rx=\"1.5\"/>", "<path d=\"M3.5 10h17M3.5 14.5h17M9.5 5v14M15 5v14\"/>"]),
];

/// The NDL bold body of `icon`, when NDL has a matching generic icon.
pub fn body(icon: Icon) -> Option<&'static [&'static str]> {
    MAPPED.iter().find(|(i, _)| *i == icon).map(|(_, b)| *b)
}

/// One flattened outline on the 24 grid: its points, whether it closes, and the indices of its
/// corners (where a round cap or join goes; the points between them come from flattened arcs).
#[derive(Debug, Default, PartialEq)]
struct Outline {
    pts: Vec<(f32, f32)>,
    corners: Vec<usize>,
    closed: bool,
}

/// One element: its outlines and whether NDL draws it solid.
#[derive(Debug, PartialEq)]
struct Element {
    outlines: Vec<Outline>,
    fill: bool,
    stroke: bool,
}

/// Paint `body` into the square `rect` in `color`. False (nothing painted) when an element is
/// outside the subset this painter reads, so the caller can draw its own icon instead.
pub fn paint(p: &Painter, rect: Rect, body: &[&str], color: Color32) -> bool {
    let Some(elements) = body.iter().map(|e| parse_element(e)).collect::<Option<Vec<_>>>() else {
        return false;
    };
    let s = rect.width() / 24.0;
    let at = |(x, y): (f32, f32)| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let w = BOLD_STROKE * s;
    for el in &elements {
        for o in &el.outlines {
            let pts: Vec<Pos2> = o.pts.iter().copied().map(at).collect();
            if el.fill && pts.len() >= 3 {
                p.add(Shape::convex_polygon(pts.clone(), color, Stroke::NONE));
            }
            if !el.stroke {
                continue;
            }
            if o.closed {
                p.add(Shape::closed_line(pts.clone(), Stroke::new(w, color)));
            } else {
                p.add(Shape::line(pts.clone(), Stroke::new(w, color)));
            }
            // Round caps and joins: a disc of the stroke's width on every corner.
            for &i in &o.corners {
                if let Some(&c) = pts.get(i) {
                    p.circle_filled(c, w / 2.0, color);
                }
            }
        }
    }
    true
}

/// The value of `name="…"` in an element's tag.
fn attr<'a>(el: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = el.find(&key)? + key.len();
    let rest = el.get(start..)?;
    rest.get(..rest.find('"')?)
}

fn num(el: &str, name: &str) -> Option<f32> {
    attr(el, name)?.trim().parse().ok().filter(|v: &f32| v.is_finite())
}

fn parse_element(el: &str) -> Option<Element> {
    let fill = attr(el, "fill").is_some_and(|f| f != "none");
    let stroke = attr(el, "stroke") != Some("none");
    let outlines = if el.starts_with("<path") {
        parse_path(attr(el, "d")?)?
    } else if el.starts_with("<circle") {
        vec![circle(num(el, "cx")?, num(el, "cy")?, num(el, "r")?)]
    } else if el.starts_with("<rect") {
        let rx = num(el, "rx").unwrap_or(0.0);
        vec![round_rect(num(el, "x")?, num(el, "y")?, num(el, "width")?, num(el, "height")?, rx)]
    } else {
        return None;
    };
    Some(Element { outlines, fill, stroke })
}

fn circle(cx: f32, cy: f32, r: f32) -> Outline {
    let n = 48;
    let pts = (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    Outline { pts, corners: Vec::new(), closed: true }
}

fn round_rect(x: f32, y: f32, w: f32, h: f32, rx: f32) -> Outline {
    let r = rx.clamp(0.0, w.min(h) / 2.0);
    let mut o = Outline { closed: true, ..Outline::default() };
    // The four corners, each a quarter arc (a single point when r is 0), clockwise from top left.
    for (cx, cy, a0) in [(x + r, y + r, 180.0_f32), (x + w - r, y + r, 270.0), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0)] {
        if r <= 0.0 {
            o.corners.push(o.pts.len());
            o.pts.push((cx, cy));
            continue;
        }
        for i in 0..=6 {
            let a = (a0 + 90.0 * i as f32 / 6.0).to_radians();
            o.pts.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    o
}

/// The numbers and command letters of a path's `d`, in order.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tok {
    Cmd(char),
    Num(f32),
}

fn tokenize(d: &str) -> Option<Vec<Tok>> {
    let mut out = Vec::new();
    let b = d.as_bytes();
    let mut i = 0;
    while let Some(&c) = b.get(i) {
        match c {
            b' ' | b',' | b'\n' | b'\t' => i += 1,
            b'A'..=b'Z' | b'a'..=b'z' if c != b'e' && c != b'E' => {
                out.push(Tok::Cmd(char::from(c)));
                i += 1;
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let start = i;
                let mut dot = false;
                if c == b'-' || c == b'+' {
                    i += 1;
                }
                while let Some(&d) = b.get(i) {
                    match d {
                        b'0'..=b'9' => i += 1,
                        b'.' if !dot => {
                            dot = true;
                            i += 1;
                        }
                        _ => break,
                    }
                }
                let v: f32 = d.get(start..i)?.parse().ok()?;
                if !v.is_finite() || i == start {
                    return None;
                }
                out.push(Tok::Num(v));
            }
            _ => return None,
        }
    }
    Some(out)
}

/// A path's outlines on the 24 grid (the subset NDL's icons use: M L H V C A Z, either case).
fn parse_path(d: &str) -> Option<Vec<Outline>> {
    let toks = tokenize(d)?;
    if toks.len() > 4096 {
        return None;
    }
    let mut outlines: Vec<Outline> = Vec::new();
    let mut cur = Outline::default();
    let (mut x, mut y) = (0.0_f32, 0.0_f32);
    let (mut sx, mut sy) = (0.0_f32, 0.0_f32);
    let mut cmd = ' ';
    let mut i = 0;
    let take = |i: &mut usize| -> Option<f32> {
        match toks.get(*i)? {
            Tok::Num(v) => {
                *i += 1;
                Some(*v)
            }
            Tok::Cmd(_) => None,
        }
    };
    let flush = |cur: &mut Outline, outlines: &mut Vec<Outline>| {
        if cur.pts.len() >= 2 {
            outlines.push(std::mem::take(cur));
        } else {
            *cur = Outline::default();
        }
    };
    while i < toks.len() {
        if let Some(Tok::Cmd(c)) = toks.get(i) {
            cmd = *c;
            i += 1;
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { (x, y) } else { (0.0, 0.0) };
        match cmd.to_ascii_uppercase() {
            'M' => {
                flush(&mut cur, &mut outlines);
                x = ox + take(&mut i)?;
                y = oy + take(&mut i)?;
                (sx, sy) = (x, y);
                cur.corners.push(0);
                cur.pts.push((x, y));
                // Further pairs after a moveto are linetos.
                cmd = if rel { 'l' } else { 'L' };
            }
            'L' => {
                x = ox + take(&mut i)?;
                y = oy + take(&mut i)?;
                cur.corners.push(cur.pts.len());
                cur.pts.push((x, y));
            }
            'H' => {
                x = ox + take(&mut i)?;
                cur.corners.push(cur.pts.len());
                cur.pts.push((x, y));
            }
            'V' => {
                y = oy + take(&mut i)?;
                cur.corners.push(cur.pts.len());
                cur.pts.push((x, y));
            }
            'A' => {
                let (rx, ry, rot) = (take(&mut i)?, take(&mut i)?, take(&mut i)?);
                let (large, sweep) = (take(&mut i)? != 0.0, take(&mut i)? != 0.0);
                let (ex, ey) = (ox + take(&mut i)?, oy + take(&mut i)?);
                arc(&mut cur.pts, (x, y), (ex, ey), rx, ry, rot, large, sweep);
                (x, y) = (ex, ey);
                cur.corners.push(cur.pts.len().saturating_sub(1));
            }
            'C' => {
                let (x1, y1) = (ox + take(&mut i)?, oy + take(&mut i)?);
                let (x2, y2) = (ox + take(&mut i)?, oy + take(&mut i)?);
                let (ex, ey) = (ox + take(&mut i)?, oy + take(&mut i)?);
                for k in 1..=12 {
                    let t = k as f32 / 12.0;
                    let u = 1.0 - t;
                    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    cur.pts.push((a * x + b * x1 + c * x2 + d * ex, a * y + b * y1 + c * y2 + d * ey));
                }
                (x, y) = (ex, ey);
                cur.corners.push(cur.pts.len().saturating_sub(1));
            }
            'Z' => {
                cur.closed = true;
                (x, y) = (sx, sy);
                flush(&mut cur, &mut outlines);
                // A segment after Z starts at the subpath's start.
                cur.corners.push(0);
                cur.pts.push((x, y));
                cmd = ' ';
                continue;
            }
            _ => return None,
        }
        if cur.pts.len() > 8192 {
            return None;
        }
    }
    flush(&mut cur, &mut outlines);
    Some(outlines)
}

/// Append the points of an SVG elliptical arc from `p0` to `p1` (the end point included),
/// converting SVG's endpoint form to a centre and angles (SVG 1.1, appendix F.6).
#[allow(clippy::too_many_arguments)]
fn arc(out: &mut Vec<(f32, f32)>, p0: (f32, f32), p1: (f32, f32), rx: f32, ry: f32, rot: f32, large: bool, sweep: bool) {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-6 || ry < 1e-6 || (p0.0 - p1.0).abs() + (p0.1 - p1.1).abs() < 1e-6 {
        out.push(p1);
        return;
    }
    let phi = rot.to_radians();
    let (cp, sp) = (phi.cos(), phi.sin());
    let (dx, dy) = ((p0.0 - p1.0) / 2.0, (p0.1 - p1.1) / 2.0);
    let x1 = cp * dx + sp * dy;
    let y1 = -sp * dx + cp * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let k = lambda.sqrt();
        rx *= k;
        ry *= k;
    }
    let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut co = if den > 0.0 { (num / den).sqrt() } else { 0.0 };
    if large == sweep {
        co = -co;
    }
    let cx1 = co * rx * y1 / ry;
    let cy1 = -co * ry * x1 / rx;
    let cx = cp * cx1 - sp * cy1 + (p0.0 + p1.0) / 2.0;
    let cy = sp * cx1 + cp * cy1 + (p0.1 + p1.1) / 2.0;
    let ang = |ux: f32, uy: f32| uy.atan2(ux);
    let t1 = ang((x1 - cx1) / rx, (y1 - cy1) / ry);
    let t2 = ang((-x1 - cx1) / rx, (-y1 - cy1) / ry);
    let tau = std::f32::consts::TAU;
    let mut dt = t2 - t1;
    if sweep && dt < 0.0 {
        dt += tau;
    } else if !sweep && dt > 0.0 {
        dt -= tau;
    }
    let n = ((dt.abs() / tau * 48.0).ceil() as usize).clamp(2, 64);
    for k in 1..=n {
        let t = t1 + dt * k as f32 / n as f32;
        let (ex, ey) = (rx * t.cos(), ry * t.sin());
        out.push((cp * ex - sp * ey + cx, sp * ex + cp * ey + cy));
    }
    if let Some(last) = out.last_mut() {
        *last = p1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mapped_body_parses() {
        assert!(MAPPED.len() > 25);
        for (icon, b) in MAPPED {
            for e in *b {
                assert!(parse_element(e).is_some(), "{icon:?}: {e}");
            }
        }
    }

    #[test]
    fn packed_numbers_split_like_svg() {
        assert_eq!(tokenize("M.7.4-1.5-2"), Some(vec![Tok::Cmd('M'), Tok::Num(0.7), Tok::Num(0.4), Tok::Num(-1.5), Tok::Num(-2.0)]));
    }

    #[test]
    fn a_quarter_arc_ends_where_svg_says() {
        let o = parse_path("M3.5 9.5h11a5.25 5.25 0 0 1 0 10.5H11").unwrap_or_default();
        let pts = &o.first().map(|o| o.pts.clone()).unwrap_or_default();
        assert_eq!(pts.first(), Some(&(3.5, 9.5)));
        assert_eq!(pts.last(), Some(&(11.0, 20.0)));
        // The half turn bulges right of x = 14.5 by its radius.
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!((max_x - 19.75).abs() < 0.1, "{max_x}");
    }

    #[test]
    fn hostile_paths_are_refused_not_panicked() {
        for d in ["M", "M1", "Q1 2 3 4", "M1 2 A", "M1e9999 2", "#"] {
            let _ = parse_path(d);
        }
        assert!(parse_path("Q1 2 3 4").is_none());
        assert!(parse_element("<ellipse cx=\"1\"/>").is_none());
    }
}
