//! Per-bubble fill-ratio + confidence scoring (P3-04).
//!
//! Inputs:
//! - The warped flat-field "ink" image (8UC1, paper=0, ink=255) produced by
//!   [`crate::threshold::flatten_to_ink`] applied to the warped canvas.
//! - The template's [`OmrTemplate`] groups, expressed in normalized `[0, 1]` coords.
//!
//! Algorithm per bubble:
//!
//! 1. Convert the bubble's normalized centre to canvas pixel coordinates.
//! 2. Sample a circular ROI of radius `BUBBLE_RADIUS_FRACTION * canvas_w`.
//! 3. Fill = `mean_ink / 255` inside the ROI. Range: `[0.0, 1.0]`. We use the
//!    bg-subtracted continuous "ink" signal (rather than an adaptive-threshold
//!    binary) because adaptive thresholding under-marks uniformly filled discs —
//!    its local mean rises with the ink and the centre of a solid-filled bubble
//!    drops below threshold (ADR `0010-confidence-band` documents the trade-off).
//! 4. Confidence is derived from the distance of the fill to the decision midpoint
//!    `0.5`, normalized so `0.0` (boundary) ⇒ confidence 0 and `0.5` (boundary) ⇒
//!    confidence 1. The grading engine's `[0.35, 0.65]` band still owns the
//!    `needs_review` decision; confidence is purely an ordering signal for the
//!    manual-review queue (P3-07).

use opencv::core::{Mat, Point, Scalar};
use opencv::{core, imgproc, prelude::*};

use shalgalt_core::domain::{BubbleReading, OmrTemplate};
use shalgalt_core::error::{AppError, AppResult};

/// Bubble sampling radius as a fraction of the warped canvas width. Calibrated for
/// the Mongolian-standard preset's 0.012 normalized bubble radius — tightly inside
/// the printed circle so the ring itself does not dominate the fill ratio.
pub const BUBBLE_RADIUS_FRACTION: f32 = 0.010;

/// Read every bubble defined in `template.groups` against `ink` and return one
/// [`BubbleReading`] per bubble. `ink` is the flat-field 8UC1 "ink density" map
/// where paper = 0 and ink ≈ 255 (typically the output of
/// [`crate::threshold::flatten_to_ink`] on the warped canvas).
pub fn read_bubbles(ink: &Mat, template: &OmrTemplate) -> AppResult<Vec<BubbleReading>> {
    let w = ink.cols();
    let h = ink.rows();
    if w <= 0 || h <= 0 {
        return Err(AppError::Internal(anyhow::anyhow!("ink canvas is empty")));
    }
    let radius_px = (BUBBLE_RADIUS_FRACTION * w as f32).round().max(2.0) as i32;

    let mut out: Vec<BubbleReading> = Vec::new();
    for group in &template.groups {
        for (idx, p) in group.bubbles.iter().enumerate() {
            let cx = (p.x * w as f32).round() as i32;
            let cy = (p.y * h as f32).round() as i32;
            let fill = sample_fill(ink, cx, cy, radius_px)?;
            out.push(BubbleReading {
                group_id: group.id.clone(),
                bubble_index: idx as u32,
                fill,
                confidence: confidence_from_fill(fill),
            });
        }
    }
    Ok(out)
}

/// Sample the mean ink density (8U values, 0=paper, 255=ink) inside the disc of
/// radius `radius_px` centred at `(cx, cy)`, normalized to `[0, 1]`. Builds a mask
/// once via `imgproc::circle` to keep the per-bubble cost constant.
fn sample_fill(ink: &Mat, cx: i32, cy: i32, radius_px: i32) -> AppResult<f32> {
    let w = ink.cols();
    let h = ink.rows();
    // Clamp the centre to the image so far-out templates do not crash sampling.
    let cx = cx.clamp(radius_px, w - radius_px - 1);
    let cy = cy.clamp(radius_px, h - radius_px - 1);

    // Build a black mask, paint a white-filled disc at the bubble centre.
    let mut mask = Mat::new_rows_cols_with_default(h, w, core::CV_8UC1, Scalar::all(0.0))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("mask alloc: {e}")))?;
    imgproc::circle(
        &mut mask,
        Point::new(cx, cy),
        radius_px,
        Scalar::all(255.0),
        -1, // filled
        imgproc::LINE_8,
        0,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("imgproc::circle: {e}")))?;

    // `mean(ink, mask)` returns the average pixel value of `ink` inside the masked
    // region. Result is in `[0, 255]` for 8UC1; we normalize to `[0, 1]`.
    let m = core::mean(ink, &mask)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("mean inside mask: {e}")))?;
    let mean_value = m[0] as f32;
    Ok((mean_value / 255.0).clamp(0.0, 1.0))
}

/// Map a fill ratio in `[0, 1]` to a confidence in `[0, 1]`.
///
/// Confidence is the normalized distance of `fill` from the band centre 0.5. The
/// minimum of `0.0` is reached at `fill = 0.5`; readings near the band edges (0.35 or
/// 0.65) score around `0.3`; readings far from the band (≤0.05 or ≥0.95) score above
/// `0.9`. The grading engine ignores this value today (it owns the decision via the
/// fixed `[0.35, 0.65]` band — see ADR `0010-confidence-band`); the manual-review
/// queue (P3-07) uses confidence to surface low-confidence sheets first.
pub fn confidence_from_fill(fill: f32) -> f32 {
    let centred = (fill - 0.5).abs(); // [0, 0.5]
    (centred * 2.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_is_zero_at_band_centre() {
        assert!((confidence_from_fill(0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn confidence_is_one_at_extremes() {
        assert!((confidence_from_fill(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((confidence_from_fill(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn confidence_is_low_inside_uncertain_band() {
        // 0.35 is the lower band edge; distance from 0.5 is 0.15 → confidence 0.30.
        let c = confidence_from_fill(0.35);
        assert!((c - 0.30).abs() < 1e-6);
        let c = confidence_from_fill(0.65);
        assert!((c - 0.30).abs() < 1e-6);
    }
}
