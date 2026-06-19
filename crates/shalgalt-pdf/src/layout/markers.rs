//! Four-corner alignment markers — the visual reference points the CV pipeline relies on
//! during the perspective-warp stage.
//!
//! [`MarkerKind::Square`] paints as a solid black square (legacy P0/P1 markers).
//! [`MarkerKind::Aruco6x6`] paints the canonical OpenCV `DICT_6X6_50` bit grid for IDs
//! 0..3 (the four corner slots of the Mongolian-standard preset, in TL/TR/BR/BL order).
//! See `docs/adr/0009-aruco-markers.md` for the rationale.

use printpdf::{Color, Op, Rgb};
use shalgalt_core::domain::{
    template::{Marker, MarkerKind},
    PaperSpec,
};

use crate::{canvas::Canvas, coords, shapes};

/// Canonical OpenCV `DICT_6X6_50` inner bit patterns for IDs 0..3.
///
/// Each entry is the 6×6 data grid (inner). `1` = white cell (no ink), `0` = black cell
/// (ink). The 1-cell black border that surrounds every ArUco marker is added at paint
/// time, not stored here.
///
/// These bytes were extracted from `cv::aruco::drawMarker` against the system OpenCV's
/// predefined dictionary so that markers we print are bit-for-bit detectable by
/// `cv::aruco::ArucoDetector` at scan time.
const ARUCO_6X6_50: [[[u8; 6]; 6]; 4] = [
    // ID 0
    [
        [0, 0, 0, 1, 1, 1],
        [1, 0, 0, 0, 1, 1],
        [1, 1, 0, 1, 1, 1],
        [0, 1, 1, 0, 0, 0],
        [0, 0, 1, 0, 1, 0],
        [1, 0, 0, 1, 1, 0],
    ],
    // ID 1
    [
        [0, 0, 0, 0, 1, 1],
        [1, 0, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1, 1],
        [1, 0, 0, 0, 1, 0],
        [0, 1, 0, 0, 0, 1],
    ],
    // ID 2
    [
        [0, 0, 0, 1, 0, 1],
        [0, 1, 1, 0, 0, 1],
        [0, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 0],
        [1, 0, 1, 0, 1, 1],
        [0, 0, 1, 1, 0, 1],
    ],
    // ID 3
    [
        [1, 1, 0, 0, 1, 0],
        [0, 1, 0, 0, 0, 1],
        [1, 0, 1, 1, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [0, 1, 1, 0, 1, 0],
        [0, 1, 1, 1, 1, 0],
    ],
];

/// Total width of an ArUco marker, in cells (1-cell black border on each side + 6×6
/// inner data).
const ARUCO_TOTAL_CELLS: usize = 8;

/// Push the four marker draw ops onto the canvas. Markers may bleed outside the printable
/// margin, so positioning uses [`coords::to_page_mm`] (margin-agnostic).
///
/// `markers[i]` is rendered with marker ID `ids[i]` for [`MarkerKind::Aruco6x6`]; the
/// canonical TL/TR/BR/BL order maps slot `i` to ArUco ID `i`.
pub fn draw(canvas: &mut Canvas, markers: &[Marker; 4], paper: &PaperSpec) {
    canvas.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });

    for (slot, marker) in markers.iter().enumerate() {
        let (cx, cy) =
            coords::to_page_mm(marker.position.x as f64, marker.position.y as f64, paper);
        // The marker's `size` is normalized; convert to mm using the shorter page edge.
        // `half` is the half-side of the full marker square (border included).
        let half = marker.size as f64 * paper.width_mm.min(paper.height_mm) * 0.5;

        match marker.kind {
            MarkerKind::Square => {
                canvas.push(Op::DrawPolygon {
                    polygon: shapes::square_polygon(cx, cy, half),
                });
            }
            MarkerKind::Aruco6x6 { ids } => {
                let id = ids[slot] as usize;
                draw_aruco_6x6(canvas, cx, cy, half, id);
            }
        }
    }
}

/// Paint a single ArUco DICT_6X6_50 marker.
///
/// Layout: 8×8 cell grid (1-cell black border + 6×6 inner data). The outer black ring
/// is drawn as a single border square minus an inner cut-out; the inner data grid is
/// drawn cell by cell, painting one black square per `0` bit.
fn draw_aruco_6x6(canvas: &mut Canvas, cx_mm: f64, cy_mm: f64, half_mm: f64, id: usize) {
    // Each cell is `cell_mm` wide. Total marker width = 8 cells = 2 * half_mm.
    let cell_mm = (2.0 * half_mm) / ARUCO_TOTAL_CELLS as f64;

    // 1) Paint the full marker as black. Inner white cells will be punched out below
    //    via stacked white squares (printpdf does not expose path subtraction; layering
    //    a white-filled square over a black background is the simplest stable
    //    approach in our printpdf 0.9 pipeline).
    canvas.push(Op::DrawPolygon {
        polygon: shapes::square_polygon(cx_mm, cy_mm, half_mm),
    });

    // 2) Switch to white fill for the inner-data white cells.
    canvas.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            icc_profile: None,
        }),
    });

    let pattern = if id < ARUCO_6X6_50.len() {
        &ARUCO_6X6_50[id]
    } else {
        // Defensive: out-of-range IDs paint as solid black (id 0 with all bits zero).
        // This branch is unreachable in our preset (IDs 0..3 only).
        &ARUCO_6X6_50[0]
    };

    // Inner 6×6 grid sits one cell inside the outer square.
    // Top-left of the marker square in PDF coordinates: (cx - half, cy + half).
    // PDF y axis points up; row 0 of the bit pattern is the TOP row of the marker.
    let top_left_x = cx_mm - half_mm;
    let top_y = cy_mm + half_mm;
    for (row, bits) in pattern.iter().enumerate() {
        for (col, &bit) in bits.iter().enumerate() {
            if bit == 1 {
                // White cell. Paint a white-filled square one cell inside the border.
                let cell_x = top_left_x + (col as f64 + 1.0) * cell_mm;
                let cell_top_y = top_y - (row as f64 + 1.0) * cell_mm;
                let cell_cx = cell_x + cell_mm * 0.5;
                let cell_cy = cell_top_y - cell_mm * 0.5;
                canvas.push(Op::DrawPolygon {
                    polygon: shapes::square_polygon(cell_cx, cell_cy, cell_mm * 0.5),
                });
            }
        }
    }

    // 3) Restore black fill for whatever the next caller wants to paint.
    canvas.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use shalgalt_core::domain::template::TemplatePoint;

    fn paper() -> PaperSpec {
        PaperSpec::A4_PORTRAIT
    }

    fn marker(id: &str, x: f32, y: f32, kind: MarkerKind) -> Marker {
        Marker {
            id: id.into(),
            position: TemplatePoint { x, y },
            size: 0.04,
            kind,
        }
    }

    fn count_polygons(canvas: &Canvas) -> usize {
        canvas
            .ops()
            .iter()
            .filter(|op| matches!(op, Op::DrawPolygon { .. }))
            .count()
    }

    #[test]
    fn square_marker_emits_one_polygon_per_marker() {
        let mut c = Canvas::new();
        let markers = [
            marker("tl", 0.05, 0.05, MarkerKind::Square),
            marker("tr", 0.95, 0.05, MarkerKind::Square),
            marker("br", 0.95, 0.95, MarkerKind::Square),
            marker("bl", 0.05, 0.95, MarkerKind::Square),
        ];
        draw(&mut c, &markers, &paper());
        assert_eq!(count_polygons(&c), 4);
    }

    #[test]
    fn aruco_marker_emits_outer_plus_inner_cells() {
        // Each ArUco marker emits 1 outer black square + N white inner cells where N
        // is the count of `1` bits in the 6×6 pattern. We count the total polygon ops
        // for ID 0 and compare against the expected count derived from the pattern.
        let mut c = Canvas::new();
        let markers = [
            marker("tl", 0.05, 0.05, MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] }),
            marker("tr", 0.95, 0.05, MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] }),
            marker("br", 0.95, 0.95, MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] }),
            marker("bl", 0.05, 0.95, MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] }),
        ];
        draw(&mut c, &markers, &paper());

        let total_ones: usize = ARUCO_6X6_50
            .iter()
            .map(|grid| grid.iter().flatten().filter(|&&b| b == 1).count())
            .sum();
        // 4 outer squares + total white inner cells across IDs 0..3.
        assert_eq!(count_polygons(&c), 4 + total_ones);
    }
}
