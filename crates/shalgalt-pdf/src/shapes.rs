//! Polygon approximations for shapes (circles, squares) that `printpdf` 0.9 does not
//! expose as primitives.
//!
//! The single-circle path uses a four-quadrant cubic Bézier with the standard
//! `kappa = 0.5522847498…`. `printpdf::Polygon` flags every `LinePoint` as anchor or
//! control via `bezier: bool`. Each quadrant emits `(anchor, ctrl₁, ctrl₂, anchor)`; we
//! register the starting anchor once and then add 12 more points in the
//! `(ctrl₁, ctrl₂, anchor) × 4` pattern.

use printpdf::{LinePoint, Mm, PaintMode, Point, Polygon, PolygonRing, WindingOrder};

/// Standard kappa constant used by the four-quadrant cubic Bézier circle approximation.
const KAPPA: f64 = 0.5522847498307936;

/// Approximate a circle of radius `r` (mm) centered at `(cx, cy)` with a polygon.
///
/// `paint` selects the fill / stroke / fill-stroke mode that the PDF will paint.
pub fn circle_polygon(cx: f64, cy: f64, r: f64, paint: PaintMode) -> Polygon {
    let cv = r * KAPPA;

    // Cardinal anchors in counter-clockwise order, starting at the top (PDF y axis goes up).
    let top = (cx, cy + r);
    let left = (cx - r, cy);
    let bottom = (cx, cy - r);
    let right = (cx + r, cy);

    let pts: [(f64, f64, bool); 13] = [
        (top.0, top.1, false),
        // top → left
        (top.0 - cv, top.1, true),
        (left.0, left.1 + cv, true),
        (left.0, left.1, false),
        // left → bottom
        (left.0, left.1 - cv, true),
        (bottom.0 - cv, bottom.1, true),
        (bottom.0, bottom.1, false),
        // bottom → right
        (bottom.0 + cv, bottom.1, true),
        (right.0, right.1 - cv, true),
        (right.0, right.1, false),
        // right → top
        (right.0, right.1 + cv, true),
        (top.0 + cv, top.1, true),
        (top.0, top.1, false),
    ];

    let points = pts
        .iter()
        .map(|(x, y, b)| LinePoint {
            p: Point::new(Mm(*x as f32), Mm(*y as f32)),
            bezier: *b,
        })
        .collect();

    Polygon {
        rings: vec![PolygonRing { points }],
        mode: paint,
        winding_order: WindingOrder::NonZero,
    }
}

/// Solid square polygon of side `2 * half` mm, centered at `(cx, cy)`. Used by corner
/// markers and the variant label box.
pub fn square_polygon(cx: f64, cy: f64, half: f64) -> Polygon {
    let pts = [
        (cx - half, cy - half),
        (cx + half, cy - half),
        (cx + half, cy + half),
        (cx - half, cy + half),
    ];
    Polygon {
        rings: vec![PolygonRing {
            points: pts
                .iter()
                .map(|(x, y)| LinePoint {
                    p: Point::new(Mm(*x as f32), Mm(*y as f32)),
                    bezier: false,
                })
                .collect(),
        }],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    }
}
