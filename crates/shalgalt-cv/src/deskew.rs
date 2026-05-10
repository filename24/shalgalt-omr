//! Hough-line auto-deskew (P3-03).
//!
//! Pre-warp rotation correction. After binarization but before marker-driven
//! perspective warp, we estimate the dominant horizontal-line angle from
//! `HoughLinesP` and rotate the page back to vertical when the estimate is small.
//!
//! Limits:
//! - Correction is capped at ±10°. Larger tilts almost always indicate the page is
//!   sideways or the markers are out of frame; the marker-driven homography handles
//!   those cases (master plan §6.5: "deskew is marker-driven").
//! - When fewer than `MIN_LINES` Hough lines are found, no rotation is applied —
//!   noisy fixtures should not produce spurious tilts.

use opencv::core::{Mat, Point2f, Scalar, Size, Vec4i, Vector};
use opencv::{core, imgproc, prelude::*};

use shalgalt_core::error::{AppError, AppResult};

/// Cap on the rotation correction. Anything outside this band is left to the
/// marker-driven homography.
pub const MAX_DESKEW_DEGREES: f64 = 10.0;

/// Minimum Hough line count needed before we trust the estimate.
pub const MIN_LINES: usize = 8;

/// Hough threshold (vote count) and minimum line length, expressed as a fraction of
/// the image's shorter side. Tuned for 200-DPI raster.
const HOUGH_VOTES_PER_KILO_PX: f64 = 35.0;
const MIN_LINE_LEN_FRACTION: f64 = 0.10;
const MAX_LINE_GAP_PX: f64 = 6.0;

/// Estimate the dominant near-horizontal angle of `binary` (8UC1, ink=255) and rotate
/// the **original** `gray` image by `-angle` so subsequent processing sees a level page.
///
/// Returns `(rotated_gray, applied_angle_deg)`. `applied_angle_deg` is `0.0` when no
/// correction was applied.
pub fn deskew(gray: &Mat, binary: &Mat) -> AppResult<(Mat, f64)> {
    let estimate = estimate_angle_deg(binary)?;
    if estimate.abs() < 0.25 || estimate.abs() > MAX_DESKEW_DEGREES {
        // Inside the noise floor or outside our trust window; pass through.
        return Ok((gray.clone(), 0.0));
    }
    let rotated = rotate(gray, -estimate)?;
    Ok((rotated, estimate))
}

/// Run `HoughLinesP` and return the median angle of near-horizontal lines, in degrees.
/// Returns `0.0` when fewer than `MIN_LINES` near-horizontal lines are found.
pub fn estimate_angle_deg(binary: &Mat) -> AppResult<f64> {
    let cols = binary.cols() as f64;
    let rows = binary.rows() as f64;
    let short = cols.min(rows);

    let votes = ((short / 1000.0) * HOUGH_VOTES_PER_KILO_PX).max(50.0) as i32;
    let min_len = (short * MIN_LINE_LEN_FRACTION).max(40.0);

    let mut lines: Vector<Vec4i> = Vector::new();
    imgproc::hough_lines_p(
        binary,
        &mut lines,
        1.0,
        std::f64::consts::PI / 180.0,
        votes,
        min_len,
        MAX_LINE_GAP_PX,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("hough_lines_p: {e}")))?;

    let mut angles: Vec<f64> = Vec::new();
    for i in 0..lines.len() {
        let l = lines
            .get(i)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("hough_lines_p: index {i}: {e}")))?;
        let dx = (l[2] - l[0]) as f64;
        let dy = (l[3] - l[1]) as f64;
        let mut deg = dy.atan2(dx).to_degrees();
        // Fold to ±90.
        if deg > 90.0 {
            deg -= 180.0;
        }
        if deg < -90.0 {
            deg += 180.0;
        }
        // Keep only "near horizontal" lines (within ±20° of 0°). Vertical bubble-grid
        // edges give |deg| ≈ 90 and would skew the median.
        if deg.abs() <= 20.0 {
            angles.push(deg);
        }
    }

    if angles.len() < MIN_LINES {
        return Ok(0.0);
    }
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = angles[angles.len() / 2];
    Ok(median)
}

fn rotate(src: &Mat, angle_deg: f64) -> AppResult<Mat> {
    let centre = Point2f::new(src.cols() as f32 * 0.5, src.rows() as f32 * 0.5);
    let m = imgproc::get_rotation_matrix_2d(centre, angle_deg, 1.0)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("get_rotation_matrix_2d: {e}")))?;
    let mut dst = Mat::default();
    imgproc::warp_affine(
        src,
        &mut dst,
        &m,
        Size::new(src.cols(), src.rows()),
        imgproc::INTER_LINEAR,
        core::BORDER_CONSTANT,
        Scalar::all(255.0),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("warp_affine: {e}")))?;
    Ok(dst)
}
