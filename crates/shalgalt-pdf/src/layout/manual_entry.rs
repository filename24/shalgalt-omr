//! Handwriting backup slots for student-ID rows.
//!
//! When OMR detection fails (smudged bubble, torn corner marker), graders fall back to
//! the handwritten value the student wrote next to the bubble row. This module draws one
//! short underline to the LEFT of every `BubbleKind::StudentId` group so the student has
//! somewhere to put a pen mark.
//!
//! No interaction with the bubbles themselves — the line sits in the margin between the
//! page edge and the row label.

use shalgalt_core::domain::{
    template::{BubbleGroup, BubbleKind},
    PaperSpec,
};

use crate::{canvas::Canvas, coords, style::BubbleStyle};

/// Underline thickness in mm.
const UNDERLINE_THICKNESS_MM: f64 = 0.3;

/// Gap (mm) between the underline's right edge and the first bubble's left edge.
/// Matches `LABEL_BUBBLE_GAP_MM` in `layout::labels` so the underline ends at the same
/// X column where a Section-1 row label ends.
const UNDERLINE_BUBBLE_GAP_MM: f64 = 1.5;

/// Underline length in mm. Matched to the visual envelope of a 2–3 character Section-1
/// row label ("1" … "70") so cipher rows and answer rows occupy the same left-edge
/// grid column — info zone and answer zone start at the same visible X.
const UNDERLINE_LENGTH_MM: f64 = 5.0;

/// Draw one underline per student-ID row in `groups`.
pub fn draw(canvas: &mut Canvas, groups: &[BubbleGroup], paper: &PaperSpec, style: &BubbleStyle) {
    let r_mm = style.diameter_mm * 0.5;
    let mut active_section = false;

    for group in groups {
        if !matches!(group.kind, BubbleKind::StudentId) {
            continue;
        }
        let Some(anchor) = group.bubbles.first() else {
            continue;
        };
        let (cx_mm, cy_mm) = coords::project(anchor.x as f64, anchor.y as f64, paper);

        // Underline ends one `UNDERLINE_BUBBLE_GAP_MM` short of the first bubble (same
        // gap a Section-1 row label uses), then runs `UNDERLINE_LENGTH_MM` to the left
        // — so its X envelope matches a typical row-label visual width.
        let line_end_x = cx_mm.0 as f64 - r_mm - UNDERLINE_BUBBLE_GAP_MM;
        let line_start_x = (line_end_x - UNDERLINE_LENGTH_MM).max(paper.margin_mm);
        // Place the underline at the bubble's lower edge so the handwritten digit
        // baseline sits on the same row as the bubble centres.
        let line_y = cy_mm.0 as f64 - r_mm;

        if line_end_x <= line_start_x {
            continue;
        }

        if !active_section {
            canvas.set_stroke_black(UNDERLINE_THICKNESS_MM);
            active_section = true;
        }

        canvas.hline(line_start_x, line_end_x, line_y);
    }
}
