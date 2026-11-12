//! Draws NDL icons with egui: a small reader for the SVG subset the set uses (paths with
//! M L H V C A Z in either case, rects with rx, circles; fills, `fill-opacity` and
//! `stroke-dasharray`), flattened to egui shapes with round caps and joins.
//!
//! A filled element is filled even-odd (an outline inside another is a hole in it): as a convex
//! polygon when it is one, else triangulated, since egui only fills convex polygons.

use egui::{epaint::Mesh, pos2, Color32, Painter, Pos2, Rect, Shape, Stroke};

use super::{Icon, BOLD_STROKE, STROKE};

/// How to draw an icon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    /// The ink: every element, or every element but the accented ones when `accent` is set.
    pub ink: Color32,
    /// The colour of the elements the icon marks as accented (CAD tools: the geometry a tool
    /// creates). `None` draws the whole icon in `ink`.
    pub accent: Option<Color32>,
    /// The bold style (Nodalia apps' interface icons) instead of the regular one.
    pub bold: bool,
    /// A stroke width in points that replaces the style's own ([`STROKE`] or [`BOLD_STROKE`] on
    /// the 24 grid), for icons that must stay fine at large sizes: CAD tools in a tool palette,
    /// where a stroke that grows with the icon clogs the geometry. `None` keeps the style's.
    pub stroke: Option<f32>,
}

impl Style {
    /// The whole icon in one colour, in the regular style.
    pub fn regular(ink: Color32) -> Self {
        Style {
            ink,
            accent: None,
            bold: false,
            stroke: None,
        }
    }

    /// The whole icon in one colour, in the bold style.
    pub fn bold(ink: Color32) -> Self {
        Style {
            ink,
            accent: None,
            bold: true,
            stroke: None,
        }
    }
}

/// Paint `icon` into the square `rect`. False (nothing painted) when one of its elements is
/// outside the subset this reader understands.
pub fn paint(p: &Painter, rect: Rect, icon: &Icon, style: Style) -> bool {
    let body = if style.bold {
        icon.bold_body()
    } else {
        icon.body
    };
    let Some(elements) = body
        .iter()
        .map(|e| parse_element(e))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let s = rect.width() / 24.0;
    let at = |(x, y): (f32, f32)| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let w = style
        .stroke
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(if style.bold { BOLD_STROKE } else { STROKE } * s);
    for (i, el) in elements.iter().enumerate() {
        let color = match style.accent {
            Some(accent) if icon.accent.contains(&i) => accent,
            _ => style.ink,
        };
        if el.fill {
            // Without a stroke over it, a triangulated fill's edge needs a hairline to be smooth.
            let edge = (!el.stroke && el.fill_opacity >= 1.0).then(|| Stroke::new(0.5 * s, color));
            fill(
                p,
                &el.outlines,
                &at,
                color.gamma_multiply(el.fill_opacity),
                edge,
            );
        }
        for o in &el.outlines {
            let pts: Vec<Pos2> = o.pts.iter().copied().map(at).collect();
            if !el.stroke {
                continue;
            }
            let stroke = Stroke::new(w, color);
            if let Some((dash, gap)) = el.dash {
                let mut line = pts.clone();
                if o.closed {
                    line.extend(pts.first().copied());
                }
                p.extend(Shape::dashed_line(&line, stroke, dash * s, gap * s));
                continue;
            }
            if o.closed {
                p.add(Shape::closed_line(pts.clone(), stroke));
            } else {
                p.add(Shape::line(pts.clone(), stroke));
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

/// One flattened outline on the 24 grid: its points, whether it closes, and the indices of its
/// corners (where a round cap or join goes; the points between them come from flattened arcs).
#[derive(Debug, Default, PartialEq)]
struct Outline {
    pts: Vec<(f32, f32)>,
    corners: Vec<usize>,
    closed: bool,
}

/// One element: its outlines and how it is inked.
#[derive(Debug, PartialEq)]
struct Element {
    outlines: Vec<Outline>,
    fill: bool,
    fill_opacity: f32,
    stroke: bool,
    dash: Option<(f32, f32)>,
}
type Pt = (f32, f32);

/// Fill an element's outlines even-odd: an outline inside another is a hole in that one.
fn fill(
    p: &Painter,
    outlines: &[Outline],
    at: &impl Fn(Pt) -> Pos2,
    color: Color32,
    edge: Option<Stroke>,
) {
    for (poly, holes, outer) in regions(outlines) {
        let screen: Vec<Pos2> = poly.iter().copied().map(at).collect();
        if !holes && convex(&poly) {
            p.add(Shape::convex_polygon(screen, color, Stroke::NONE));
            continue;
        }
        let mut mesh = Mesh::default();
        for &v in &screen {
            mesh.colored_vertex(v, color);
        }
        for [a, b, c] in triangulate(&poly) {
            mesh.add_triangle(a, b, c);
        }
        p.add(Shape::mesh(mesh));
        if let Some(edge) = edge {
            p.add(Shape::closed_line(
                outer.iter().copied().map(at).collect(),
                edge,
            ));
        }
    }
}

/// The filled regions of an element's outlines under the even-odd rule: an outline nested at an
/// even depth is filled, with the outlines directly inside it bridged in as holes. Each region is
/// its positively oriented outline, whether it has holes, and its outer outline alone.
fn regions(outlines: &[Outline]) -> Vec<(Vec<Pt>, bool, &[Pt])> {
    let usable: Vec<&[Pt]> = outlines
        .iter()
        .map(|o| o.pts.as_slice())
        .filter(|pts| pts.len() >= 3)
        .collect();
    let contains = |j: usize, k: usize| {
        let first = usable.get(k).and_then(|pts| pts.first().copied());
        j != k && first.is_some_and(|f| usable.get(j).is_some_and(|o| inside(f, o)))
    };
    let depth: Vec<usize> = (0..usable.len())
        .map(|k| (0..usable.len()).filter(|&j| contains(j, k)).count())
        .collect();
    let depth_of = |k: usize| depth.get(k).copied().unwrap_or(0);
    let mut out = Vec::new();
    for (k, &outer) in usable.iter().enumerate() {
        if depth_of(k) % 2 == 1 {
            continue;
        }
        let mut poly = oriented(outer, true);
        let mut holes = false;
        for (h, pts) in usable.iter().enumerate() {
            if depth_of(h) == depth_of(k) + 1 && contains(k, h) {
                bridge(&mut poly, &oriented(pts, false));
                holes = true;
            }
        }
        out.push((poly, holes, outer));
    }
    out
}

/// Twice the signed area; positive when the outline turns the way `cross` calls convex.
fn area(pts: &[Pt]) -> f32 {
    let next = pts.iter().cycle().skip(1);
    pts.iter()
        .zip(next)
        .map(|(a, b)| a.0 * b.1 - b.0 * a.1)
        .sum()
}

fn oriented(pts: &[Pt], positive: bool) -> Vec<Pt> {
    let mut v = pts.to_vec();
    if (area(&v) > 0.0) != positive {
        v.reverse();
    }
    v
}

fn cross(a: Pt, b: Pt, c: Pt) -> f32 {
    (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)
}

/// Whether a positively oriented outline never turns the other way.
fn convex(pts: &[Pt]) -> bool {
    let n = pts.len();
    (0..n).all(|i| {
        let at = |k: usize| pts.get((i + k) % n).copied().unwrap_or_default();
        cross(at(0), at(1), at(2)) >= -1e-4
    })
}

/// Even-odd point in polygon.
fn inside(q: Pt, pts: &[Pt]) -> bool {
    let next = pts.iter().cycle().skip(1);
    pts.iter()
        .zip(next)
        .filter(|(a, b)| {
            (a.1 > q.1) != (b.1 > q.1) && q.0 < a.0 + (q.1 - a.1) * (b.0 - a.0) / (b.1 - a.1)
        })
        .count()
        % 2
        == 1
}

/// Join a hole (oriented against `poly`) into `poly` through its nearest vertex pair, so the
/// two become one outline that ear clipping can fill.
fn bridge(poly: &mut Vec<Pt>, hole: &[Pt]) {
    let d = |a: Pt, b: Pt| (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2);
    let best = poly
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| hole.iter().enumerate().map(move |(j, &b)| (d(a, b), i, j)))
        .min_by(|x, y| x.0.total_cmp(&y.0));
    let Some((_, i, j)) = best else {
        return;
    };
    let ring = hole.iter().cycle().skip(j).take(hole.len() + 1).copied();
    let tail = poly.split_off(i);
    poly.extend(tail.first().copied());
    poly.extend(ring);
    poly.extend(tail);
}

/// Ear clipping of a positively oriented simple outline (bridged holes included).
fn triangulate(pts: &[Pt]) -> Vec<[u32; 3]> {
    let mut idx: Vec<usize> = (0..pts.len()).collect();
    let mut out = Vec::new();
    let pt = |i: usize| pts.get(i).copied().unwrap_or_default();
    let mut guard = pts.len() * pts.len() + 8;
    while idx.len() >= 3 && guard > 0 {
        guard -= 1;
        let n = idx.len();
        let corner = |k: usize| {
            let get = |o: usize| idx.get((k + o) % n).copied().unwrap_or_default();
            (get(n - 1), get(0), get(1))
        };
        let ear = (0..n).find(|&k| {
            let (a, b, c) = corner(k);
            let (pa, pb, pc) = (pt(a), pt(b), pt(c));
            cross(pa, pb, pc) > 1e-6
                && idx.iter().all(|&o| {
                    let q = pt(o);
                    q == pa || q == pb || q == pc || !in_triangle(q, pa, pb, pc)
                })
        });
        // No ear means only slivers are left: drop the flattest corner and carry on.
        let k = ear.unwrap_or_else(|| {
            (0..n)
                .min_by(|&x, &y| {
                    let f = |k| {
                        let (a, b, c) = corner(k);
                        cross(pt(a), pt(b), pt(c)).abs()
                    };
                    f(x).total_cmp(&f(y))
                })
                .unwrap_or(0)
        });
        let (a, b, c) = corner(k);
        if ear.is_some() {
            if let (Ok(a), Ok(b), Ok(c)) = (u32::try_from(a), u32::try_from(b), u32::try_from(c)) {
                out.push([a, b, c]);
            }
        }
        idx.remove(k);
    }
    out
}

fn in_triangle(q: Pt, a: Pt, b: Pt, c: Pt) -> bool {
    cross(a, b, q) >= 0.0 && cross(b, c, q) >= 0.0 && cross(c, a, q) >= 0.0
}

/// The value of `name="…"` in an element's tag.
fn attr<'a>(el: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = el.find(&key)? + key.len();
    let rest = el.get(start..)?;
    rest.get(..rest.find('"')?)
}

fn num(el: &str, name: &str) -> Option<f32> {
    attr(el, name)?
        .trim()
        .parse()
        .ok()
        .filter(|v: &f32| v.is_finite())
}

fn parse_element(el: &str) -> Option<Element> {
    let fill = attr(el, "fill").is_some_and(|f| f != "none");
    let stroke = attr(el, "stroke") != Some("none");
    let fill_opacity = num(el, "fill-opacity").unwrap_or(1.0).clamp(0.0, 1.0);
    let dash = attr(el, "stroke-dasharray").and_then(|d| {
        let mut n = d
            .split([' ', ','])
            .filter(|t| !t.is_empty())
            .map(|t| t.parse::<f32>().ok().filter(|v| v.is_finite() && *v > 0.0));
        Some((n.next()??, n.next()??))
    });
    let outlines = if el.starts_with("<path") {
        parse_path(attr(el, "d")?)?
    } else if el.starts_with("<circle") {
        vec![circle(num(el, "cx")?, num(el, "cy")?, num(el, "r")?)]
    } else if el.starts_with("<rect") {
        let rx = num(el, "rx").unwrap_or(0.0);
        vec![round_rect(
            num(el, "x")?,
            num(el, "y")?,
            num(el, "width")?,
            num(el, "height")?,
            rx,
        )]
    } else {
        return None;
    };
    Some(Element {
        outlines,
        fill,
        fill_opacity,
        stroke,
        dash,
    })
}

fn circle(cx: f32, cy: f32, r: f32) -> Outline {
    let n = 48;
    let pts = (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    Outline {
        pts,
        corners: Vec::new(),
        closed: true,
    }
}

fn round_rect(x: f32, y: f32, w: f32, h: f32, rx: f32) -> Outline {
    let r = rx.clamp(0.0, w.min(h) / 2.0);
    let mut o = Outline {
        closed: true,
        ..Outline::default()
    };
    // The four corners, each a quarter arc (a single point when r is 0), clockwise from top left.
    for (cx, cy, a0) in [
        (x + r, y + r, 180.0_f32),
        (x + w - r, y + r, 270.0),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, 90.0),
    ] {
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

/// A path's outlines on the 24 grid (the subset NDL's icons use: M L H V C S A Z, either case).
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
    // The second control point of the last C or S, which an S reflects.
    let mut last_c2: Option<(f32, f32)> = None;
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
        if !matches!(cmd, 'C' | 'c' | 'S' | 's') {
            last_c2 = None;
        }
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
                // The flags are 0 or 1.
                let (large, sweep) = (take(&mut i)? > 0.5, take(&mut i)? > 0.5);
                let (ex, ey) = (ox + take(&mut i)?, oy + take(&mut i)?);
                arc(&mut cur.pts, (x, y), (ex, ey), rx, ry, rot, large, sweep);
                (x, y) = (ex, ey);
                cur.corners.push(cur.pts.len().saturating_sub(1));
            }
            'C' | 'S' => {
                let (x1, y1) = if cmd.eq_ignore_ascii_case(&'C') {
                    (ox + take(&mut i)?, oy + take(&mut i)?)
                } else {
                    last_c2.map_or((x, y), |(cx, cy)| (2.0 * x - cx, 2.0 * y - cy))
                };
                let (x2, y2) = (ox + take(&mut i)?, oy + take(&mut i)?);
                let (ex, ey) = (ox + take(&mut i)?, oy + take(&mut i)?);
                for k in 1..=12 {
                    let t = k as f32 / 12.0;
                    let u = 1.0 - t;
                    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    cur.pts.push((
                        a * x + b * x1 + c * x2 + d * ex,
                        a * y + b * y1 + c * y2 + d * ey,
                    ));
                }
                (x, y) = (ex, ey);
                cur.corners.push(cur.pts.len().saturating_sub(1));
                last_c2 = Some((x2, y2));
                if cur.pts.len() > 8192 {
                    return None;
                }
                continue;
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
fn arc(
    out: &mut Vec<(f32, f32)>,
    p0: (f32, f32),
    p1: (f32, f32),
    rx: f32,
    ry: f32,
    rot: f32,
    large: bool,
    sweep: bool,
) {
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
    fn every_icon_in_the_set_parses() {
        for icon in super::super::all() {
            for e in icon.body.iter().chain(icon.bold_body()) {
                assert!(parse_element(e).is_some(), "{}: {e}", icon.name);
            }
        }
    }

    #[test]
    fn packed_numbers_split_like_svg() {
        assert_eq!(
            tokenize("M.7.4-1.5-2"),
            Some(vec![
                Tok::Cmd('M'),
                Tok::Num(0.7),
                Tok::Num(0.4),
                Tok::Num(-1.5),
                Tok::Num(-2.0)
            ])
        );
    }

    #[test]
    fn a_quarter_arc_ends_where_svg_says() {
        let o = parse_path("M3.5 9.5h11a5.25 5.25 0 0 1 0 10.5H11").unwrap_or_default();
        let pts = &o.first().map(|o| o.pts.clone()).unwrap_or_default();
        assert_eq!(pts.first(), Some(&(3.5, 9.5)));
        assert_eq!(pts.last(), Some(&(11.0, 20.0)));
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!((max_x - 19.75).abs() < 0.1, "{max_x}");
    }

    #[test]
    fn dashes_and_soft_fills_are_read() {
        let e = parse_element(r#"<path d="M12 3.5v17" stroke-dasharray="2 3"/>"#).unwrap();
        assert_eq!(e.dash, Some((2.0, 3.0)));
        let e = parse_element(
            r#"<path d="M15 7l5.5 10H15z" fill="currentColor" fill-opacity="0.18"/>"#,
        )
        .unwrap();
        assert!(e.fill && (e.fill_opacity - 0.18).abs() < 1e-6);
    }

    fn covered(poly: &[Pt]) -> f32 {
        triangulate(poly)
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| poly[i as usize]);
                cross(a, b, c).abs() / 2.0
            })
            .sum()
    }

    #[test]
    fn concave_fills_keep_their_shape() {
        let mut concave = 0;
        for icon in super::super::all() {
            for e in icon.body.iter().chain(icon.bold_body()) {
                let el = parse_element(e).unwrap();
                if !el.fill || el.outlines.len() != 1 {
                    continue;
                }
                let poly = oriented(&el.outlines[0].pts, true);
                concave += usize::from(!convex(&poly));
                let want = area(&poly) / 2.0;
                assert!(
                    (covered(&poly) - want).abs() < 0.02 * want.max(1.0),
                    "{}",
                    icon.name
                );
            }
        }
        // The bold star, heart, folder, bell... are not convex.
        assert!(concave >= 8, "{concave}");
    }

    fn filled_at(outlines: &[Outline], q: Pt) -> bool {
        regions(outlines).iter().any(|(poly, _, _)| {
            triangulate(poly).iter().any(|t| {
                let [a, b, c] = t.map(|i| poly[i as usize]);
                in_triangle(q, a, b, c)
            })
        })
    }

    #[test]
    fn a_hole_stays_open() {
        let disc = super::super::get("disc").unwrap();
        let el = disc
            .bold_body()
            .iter()
            .filter_map(|e| parse_element(e))
            .find(|el| el.fill && el.outlines.len() == 2)
            .unwrap();
        let (outer, hole) = (&el.outlines[0].pts, &el.outlines[1].pts);
        let want = (area(outer).abs() - area(hole).abs()) / 2.0;
        let got: f32 = regions(&el.outlines)
            .iter()
            .map(|(p, _, _)| covered(p))
            .sum();
        assert!((got - want).abs() < 0.02 * want, "{got} vs {want}");
        assert!(!filled_at(&el.outlines, (12.0, 12.0)));
        assert!(filled_at(&el.outlines, (12.0, 6.0)));
    }

    #[test]
    fn an_island_inside_a_hole_is_filled() {
        let squares = parse_path("M2 2h20v20H2zM6 6h12v12H6zM10 10h4v4h-4z").unwrap();
        assert_eq!(squares.len(), 3);
        assert!(filled_at(&squares, (4.0, 12.0)));
        assert!(!filled_at(&squares, (8.0, 12.0)));
        assert!(filled_at(&squares, (12.0, 12.0)));
        let got: f32 = regions(&squares).iter().map(|(p, _, _)| covered(p)).sum();
        assert!((got - (400.0 - 144.0 + 16.0)).abs() < 0.5, "{got}");
    }

    #[test]
    fn hostile_paths_are_refused_not_panicked() {
        for d in ["M", "M1", "Q1 2 3 4", "M1 2 A", "M1e9999 2", "#"] {
            let _ = parse_path(d);
        }
        assert!(parse_path("Q1 2 3 4").is_none());
        assert!(parse_element("<ellipse cx=\"1\"/>").is_none());
    }

    #[test]
    fn accented_parts_take_the_accent() {
        let ctx = egui::Context::default();
        let dimension = super::super::get("dimension").unwrap();
        let style = Style {
            ink: Color32::WHITE,
            accent: Some(Color32::RED),
            bold: false,
            stroke: None,
        };
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let p = ui.ctx().layer_painter(egui::LayerId::background());
            assert!(paint(
                &p,
                Rect::from_min_size(Pos2::ZERO, egui::vec2(24.0, 24.0)),
                dimension,
                style
            ));
        });
        out.textures_delta.clear();
        let colours: Vec<Color32> = out
            .shapes
            .iter()
            .filter_map(|s| {
                if let Shape::Path(path) = &s.shape {
                    Some(path.stroke.color.clone())
                } else {
                    None
                }
            })
            .map(|c| match c {
                egui::epaint::ColorMode::Solid(c) => c,
                _ => Color32::TRANSPARENT,
            })
            .collect();
        assert!(
            colours.contains(&Color32::WHITE) && colours.contains(&Color32::RED),
            "{colours:?}"
        );
    }

    /// The widths of the strokes `paint` draws for `name` at `size` with `style`.
    fn stroke_widths(name: &str, size: f32, style: Style) -> Vec<f32> {
        let ctx = egui::Context::default();
        let icon = super::super::get(name).unwrap();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let p = ui.ctx().layer_painter(egui::LayerId::background());
            assert!(paint(
                &p,
                Rect::from_min_size(Pos2::ZERO, egui::vec2(size, size)),
                icon,
                style
            ));
        });
        out.textures_delta.clear();
        out.shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Path(path) => Some(path.stroke.width),
                Shape::LineSegment { stroke, .. } => Some(stroke.width),
                _ => None,
            })
            .filter(|w| *w > 0.6)
            .collect()
    }

    #[test]
    fn an_explicit_stroke_stays_fine_at_any_size() {
        let regular = Style::regular(Color32::WHITE);
        let fine = Style {
            stroke: Some(1.25),
            ..regular
        };
        // The style's own stroke grows with the icon: 1.75 on the grid is 2.33 pt at 32 pt.
        assert!(stroke_widths("line", 32.0, regular)
            .iter()
            .all(|w| (*w - 1.75 * 32.0 / 24.0).abs() < 1e-3));
        assert!(stroke_widths("line", 32.0, fine)
            .iter()
            .all(|w| (*w - 1.25).abs() < 1e-3));
        // A nonsense width keeps the style's.
        let bad = Style {
            stroke: Some(f32::NAN),
            ..regular
        };
        assert!(stroke_widths("line", 24.0, bad)
            .iter()
            .all(|w| (*w - 1.75).abs() < 1e-3));
    }
}
