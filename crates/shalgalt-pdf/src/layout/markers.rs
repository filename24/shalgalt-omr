//! Four-corner alignment markers — the visual reference points the CV pipeline relies on
//! during the perspective-warp stage.
//!
//! For P2-06 both [`MarkerKind::Square`] and [`MarkerKind::Aruco6x6`] paint as solid
//! squares. ArUco bit-grid stamping ships alongside the CV migration in P3-01.

use printpdf::{Color, Op, Rgb};
use shalgalt_core::domain::{
    template::{Marker, MarkerKind},
    PaperSpec,
};

use crate::{coords, shapes};

/// Push the four marker draw ops onto `ops`. Markers may bleed outside the printable
/// margin, so positioning uses [`coords::to_page_mm`] (margin-agnostic).
pub fn draw(ops: &mut Vec<Op>, markers: &[Marker; 4], paper: &PaperSpec) {
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });

    for marker in markers {
        let (cx, cy) =
            coords::to_page_mm(marker.position.x as f64, marker.position.y as f64, paper);
        // The marker's `size` is normalized; convert to mm using the shorter page edge
        // and use the half-extent below.
        let half = marker.size as f64 * paper.width_mm.min(paper.height_mm) * 0.5;

        match marker.kind {
            // TODO(#P3-01): paint the actual 6×6 ArUco bit grid for better CV alignment.
            MarkerKind::Square | MarkerKind::Aruco6x6 { .. } => {
                ops.push(Op::DrawPolygon {
                    polygon: shapes::square_polygon(cx, cy, half),
                });
            }
        }
    }
}
