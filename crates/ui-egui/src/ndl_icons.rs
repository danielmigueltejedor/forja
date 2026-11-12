//! Forja draws its icons from Nodalia Design's icon set ([`nodalia_theme::icons`]), so a
//! concept has the same icon in every Nodalia app. The generic interface icons (files, edit,
//! view, navigation, chevrons) use NDL's bold style in one ink; the CAD tools use the regular,
//! thinner stroke, with the geometry a tool creates in the accent colour.

use egui::{Color32, Painter, Rect};
use nodalia_theme::icons::{self as ndl, Style};

use crate::icons::Icon;

/// How an icon is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A generic interface icon: NDL's bold style, one ink.
    Interface,
    /// A CAD tool: NDL's regular stroke, the created geometry in the accent.
    Cad,
}

/// The NDL icon that draws `icon`, and how.
pub fn name(icon: Icon) -> (&'static str, Kind) {
    use Icon::*;
    let interface = match icon {
        New => "new",
        Open => "open",
        Save => "save",
        Undo => "undo",
        Redo => "redo",
        Plot => "print",
        Import => "import",
        Attach => "attach",
        Share => "share",
        Pan => "pan",
        Orbit => "orbit",
        ZoomExtents | ZoomFit => "fit",
        Help => "help",
        Table => "table",
        Copy => "copy",
        Rotate => "rotate",
        Erase => "delete",
        Lock => "lock",
        Unlock => "unlock",
        Sun => "sun",
        Gear => "settings",
        ChevronDown => "chevron-down",
        ChevronUp => "chevron-up",
        ChevronLeft => "chevron-left",
        ChevronRight => "chevron-right",
        Plus => "plus",
        Close => "close",
        Search => "search",
        Menu => "menu",
        ZoomIn => "zoom",
        ZoomOut => "zoom-out",
        _ => "",
    };
    if !interface.is_empty() {
        return (interface, Kind::Interface);
    }
    let cad = match icon {
        SaveAs => "save-as",
        Publish => "publish",
        PageSetup => "page-setup",
        Preview => "preview",
        Export => "export",
        ZoomWindow => "zoom-window",
        Properties => "properties",
        Blocks => "blocks",
        Layers => "layers",
        Line => "line",
        Polyline => "polyline",
        Circle => "circle",
        Arc => "arc",
        Rectangle => "rectangle",
        Polygon => "polygon",
        Ellipse => "ellipse",
        Spline => "spline",
        Point => "point",
        XLine => "x-line",
        Ray => "ray",
        Donut => "donut",
        RevCloud => "rev-cloud",
        Hatch => "hatch",
        Gradient => "gradient",
        Boundary => "closed-boundary",
        Region => "region",
        Wipeout => "wipeout",
        Text => "text",
        MText => "m-text",
        Move => "move",
        Scale => "scale",
        Mirror => "flip",
        Offset => "offset",
        Trim => "trim",
        Extend => "extend",
        Fillet => "fillet",
        Chamfer => "chamfer",
        Explode => "explode",
        Stretch => "stretch",
        ArrayRect => "pattern",
        ArrayPolar => "array-polar",
        Break => "break",
        Join => "join",
        Lengthen => "lengthen",
        MatchProp => "match-prop",
        DimLinear => "dimension",
        DimAligned => "dim-aligned",
        DimRadius => "radius",
        DimDiameter => "dim-diameter",
        DimAngular => "angle",
        DimArc => "dim-arc",
        DimOrdinate => "dim-ordinate",
        DimBaseline => "dim-baseline",
        DimContinue => "dim-continue",
        QDim => "q-dim",
        MLeader => "m-leader",
        AddLeader => "add-leader",
        RemoveLeader => "remove-leader",
        AlignLeaders => "align-leaders",
        BlockCreate => "block-create",
        Insert => "insert",
        BlockEdit => "block-edit",
        AttDef => "att-def",
        LayerProps => "layer-props",
        LayerOff => "layer-off",
        LayerIso => "layer-iso",
        LayerFreeze => "layer-freeze",
        LayerLock => "layer-lock",
        LayerMatch => "layer-match",
        MakeCurrent => "make-current",
        LayerPrev => "layer-prev",
        Bulb => "bulb",
        BulbOff => "bulb-off",
        Snowflake => "snowflake",
        Grid => "grid",
        Snap => "snap",
        Ortho => "ortho",
        Polar => "polar",
        Osnap => "osnap",
        OTrack => "o-track",
        Lineweight => "lineweight",
        Transparency => "transparency",
        Isodraft => "isodraft",
        DynInput => "dyn-input",
        QuickProps => "quick-props",
        Annotation => "annotation",
        Workspace => "workspace",
        Switcher => "switcher",
        Constraint => "constraint",
        Coincident => "coincident",
        Parallel => "parallel",
        Perpendicular => "perpendicular",
        Horizontal => "horizontal",
        Vertical => "vertical",
        Tangent => "tangent",
        Concentric => "concentric",
        Equal => "equal",
        Fix => "fix",
        Collinear => "collinear",
        Smooth => "smooth",
        Symmetric => "symmetric",
        DelConstraint => "del-constraint",
        Parameters => "parameters",
        Measure => "measure",
        Distance => "distance",
        Area => "area",
        List => "list",
        PickAdd => "pick-add",
        SelectObjects => "select-objects",
        QuickSelect => "quick-select",
        MoveTop => "move-top",
        MoveUp => "move-up",
        MoveDown => "move-down",
        MoveBottom => "move-bottom",
        TextStyle => "text-style",
        DimStyle => "dim-style",
        _ => "",
    };
    (cad, Kind::Cad)
}

/// Paint `icon` into the square `rect`: in `ink`, with the accented parts of a CAD icon in
/// `accent`. False when NDL has no such icon (nothing is painted).
pub fn paint(p: &Painter, rect: Rect, icon: Icon, ink: Color32, accent: Color32) -> bool {
    let (name, kind) = name(icon);
    let Some(def) = ndl::get(name) else {
        return false;
    };
    let style = match kind {
        Kind::Interface => Style::bold(ink),
        Kind::Cad => Style { ink, accent: Some(accent), bold: false, stroke: Some(cad_stroke(rect.width())) },
    };
    ndl::paint(p, rect, def, style)
}

/// The stroke of a CAD tool icon `size` points wide: fine at every size (1–1.4 pt). NDL's regular
/// stroke grows with the icon, so the 28–32 pt Tool Sets buttons drew at 2 pt and more, which
/// reads as bold and clogs the geometry.
fn cad_stroke(size: f32) -> f32 {
    (size * 1.25 / 24.0).clamp(1.0, 1.4)
}

/// Paint `icon` into `rect` entirely in `color` (window controls, rows of a card).
pub fn paint_ink(p: &Painter, rect: Rect, icon: Icon, color: Color32) -> bool {
    paint(p, rect, icon, color, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_forja_shows_is_in_nodalia_design() {
        for icon in crate::icons::ALL {
            let (n, _) = name(*icon);
            assert!(ndl::get(n).is_some(), "{icon:?} -> {n:?}");
        }
    }

    #[test]
    fn cad_tools_are_thin_and_two_tone_and_the_interface_is_bold() {
        assert_eq!(name(Icon::Trim), ("trim", Kind::Cad));
        assert_eq!(name(Icon::Line), ("line", Kind::Cad));
        assert_eq!(name(Icon::Save), ("save", Kind::Interface));
        assert!(ndl::get("trim").is_some_and(|d| !d.accent.is_empty()));
        // The large Tool Sets buttons stay fine, unlike NDL's regular 1.75 stroke on the 24 grid.
        assert_eq!(cad_stroke(32.0), 1.4);
        assert!(cad_stroke(32.0) < nodalia_theme::icons::STROKE * 32.0 / 24.0);
        assert_eq!(cad_stroke(12.0), 1.0);
    }
}
