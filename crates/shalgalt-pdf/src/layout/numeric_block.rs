//! Stack several [`BubbleGroup`]s vertically to form a single-digit numeric block (the
//! Шифр column or Section-2 sub-blocks).
//!
//! This module does no coordinate arithmetic of its own — each group's normalized
//! coordinates already carry its row position, so the implementation just dispatches to
//! [`crate::layout::bubble_grid::draw`] for every group. Column headers (`0–9`) and row
//! labels are emitted at the [`crate::render_template`] level via
//! [`crate::layout::labels`], so this primitive only strokes the circles.

use printpdf::FontId;
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{canvas::Canvas, layout::bubble_grid, style::BubbleStyle};

/// Render every group in `groups`, in order. Each bubble carries the digit label drawn
/// inside its circle by [`bubble_grid::draw`]. Caller emits the row labels separately
/// via [`crate::layout::labels::draw_row_label`].
pub fn draw(
    canvas: &mut Canvas,
    groups: &[&BubbleGroup],
    paper: &PaperSpec,
    style: &BubbleStyle,
    labels: &[char],
    font: &FontId,
) {
    for group in groups {
        bubble_grid::draw(canvas, group, paper, style, labels, font);
    }
}
