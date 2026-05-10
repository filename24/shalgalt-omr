//! Adaptive thresholding with illumination flattening (P3-02).
//!
//! School OMR sheets are routinely photocopied or photographed under uneven lighting,
//! which makes a global Otsu threshold unreliable. We instead:
//!
//! 1. Estimate the slowly-varying background illumination by `medianBlur`-ing the
//!    grayscale image with a large kernel.
//! 2. Subtract the background to produce a roughly flat-field image.
//! 3. Run `adaptiveThreshold(GAUSSIAN, blockSize=41, C=7)` on the result.
//!
//! Block size 41 (odd) is locked by ADR `0010-confidence-band` based on a 200-DPI
//! raster of A4: a 41-pixel kernel covers roughly 5 mm, which is wider than the
//! widest expected ink stroke (3 mm pen) but tighter than the typical bubble-grid
//! cell (~6 mm). C=7 trims faint copier specks without erasing genuine pen ink.

use opencv::core::{Mat, Scalar};
use opencv::{core, imgproc, prelude::*};

use shalgalt_core::error::{AppError, AppResult};

/// Block size of the Gaussian adaptive threshold (must be odd, ≥3). Tuned for
/// 200-DPI A4 raster — see module-level docs.
pub const ADAPTIVE_BLOCK_SIZE: i32 = 41;

/// Constant `C` subtracted from the local mean. Higher values trim more noise but
/// can miss faint marks; 7 is the locked default.
pub const ADAPTIVE_C: f64 = 7.0;

/// Median-blur kernel used to estimate the slowly-varying background illumination.
/// Large enough to smooth over bubbles and text, small enough to track real shading.
pub const BG_MEDIAN_KSIZE: i32 = 51;

/// Convert `bgr_or_rgb` (any 3-channel image) to grayscale.
pub fn to_gray(src: &Mat) -> AppResult<Mat> {
    let mut gray = Mat::default();
    if src.channels() == 1 {
        gray = src.clone();
        return Ok(gray);
    }
    imgproc::cvt_color(src, &mut gray, imgproc::COLOR_BGR2GRAY, 0)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("cvt_color BGR2GRAY: {e}")))?;
    Ok(gray)
}

/// Subtract a median-blur background estimate from `gray`, then run
/// `adaptiveThreshold(GAUSSIAN, blockSize=41, C=7)` on the result. Returns a binary
/// 8U mask where ink is white (255) and paper is black (0).
///
/// This is intended for **outline / line-art** detection (bubble rings, text,
/// printed dividers). For solid-filled-disc measurement, prefer
/// [`flatten_to_ink`] — adaptive thresholding tends to under-mark uniformly filled
/// regions because the local mean rises with the ink itself.
pub fn binarize(gray: &Mat) -> AppResult<Mat> {
    let flat = flatten_to_ink(gray)?;

    let mut bin = Mat::default();
    imgproc::adaptive_threshold(
        &flat,
        &mut bin,
        255.0,
        imgproc::ADAPTIVE_THRESH_GAUSSIAN_C,
        imgproc::THRESH_BINARY,
        ADAPTIVE_BLOCK_SIZE,
        ADAPTIVE_C,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("adaptive_threshold: {e}")))?;

    Ok(bin)
}

/// Convert `gray` to an "ink density" map where paper ≈ 0 and ink ≈ 255 — i.e.
/// simply `255 - gray`. The bubble sampler uses this signal so that uniformly
/// filled discs read as a high mean intensity. Bg-subtraction is **not** applied
/// here because a 51-pixel median blur (the kernel sized for page-level
/// illumination) preserves filled bubbles inside it (they are smaller than the
/// kernel) and erases their signal in `bg - gray`. Page-level illumination is
/// instead handled per-ROI by [`crate::bubbles::sample_fill`], which compares
/// against the surrounding paper.
pub fn flatten_to_ink(gray: &Mat) -> AppResult<Mat> {
    let mut inverted = Mat::default();
    core::bitwise_not(gray, &mut inverted, &core::no_array())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("bitwise_not: {e}")))?;
    Ok(inverted)
}

/// Average grayscale value of `mat`, useful for diagnostics when a sheet's
/// illumination is wildly outside the usable range. Returns `None` when the matrix is
/// empty.
pub fn mean_intensity(mat: &Mat) -> Option<f64> {
    let s = core::mean(mat, &core::no_array()).ok()?;
    Some(Scalar::from(s)[0])
}
