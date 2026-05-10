//! Render a single [`BubbleGroup`] as a row of labelled circles.
//!
//! Each bubble carries a 1-character label (A/B/C/D/E for multiple choice or 0–9 for
//! digits) drawn INSIDE the circle. Real OMR cards do this so the meaning of every
//! position is unambiguous to the student even if marker detection fails — see issue
//! #75 follow-up.
//!
//! Filled bubbles (the student's answer) hide the inner label; empty bubbles show it.
//! The CV pipeline thresholds the *fraction of dark pixels* inside a circle, so the
//! small printed glyph does not register as a fill.
//!
//! Acceptance defaults:
//! - 4.5 mm diameter, 0.3 mm outline, solid black.
//! - Coordinates always go through [`crate::coords::project`] (full-page mapping —
//!   identical to [`crate::coords::to_page_mm`], see that module's notes).
//! - Per-bubble text emission lives in [`Canvas::text_centered_in_circle`], which wraps
//!   each call in its own BT/ET so the text matrix never accumulates.

use printpdf::FontId;
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{canvas::Canvas, coords, style::BubbleStyle};

/// Stroke an empty circle at every `group.bubbles[i]` and paint `labels[i]` centred
/// inside the circle. An empty `labels` slice skips the label pass entirely (used by
/// tests / future blank renders).
pub fn draw(
    canvas: &mut Canvas,
    group: &BubbleGroup,
    paper: &PaperSpec,
    style: &BubbleStyle,
    labels: &[char],
    font: &FontId,
) {
    let r_mm = style.diameter_mm * 0.5;
    canvas.set_stroke_black(style.stroke_mm);

    // 1) Stroke every circle.
    for bubble in &group.bubbles {
        let (cx_mm, cy_mm) = coords::project(bubble.x as f64, bubble.y as f64, paper);
        canvas.circle_stroked(cx_mm.0 as f64, cy_mm.0 as f64, r_mm);
    }

    if labels.is_empty() {
        return;
    }

    // 2) Centred label inside each bubble. Per-call BT/ET reset is owned by Canvas.
    let limit = labels.len().min(group.bubbles.len());
    for (i, bubble) in group.bubbles.iter().take(limit).enumerate() {
        let (cx_mm, cy_mm) = coords::project(bubble.x as f64, bubble.y as f64, paper);
        canvas.text_centered_in_circle(cx_mm.0 as f64, cy_mm.0 as f64, r_mm, labels[i], font);
    }
}
