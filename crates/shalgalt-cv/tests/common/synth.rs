//! Synthetic OMR page generator for integration fixtures.
//!
//! Draws an A4-sized 8-bit grayscale page with:
//!
//! - Four ArUco DICT_6X6_50 markers (IDs 0..3 in TL/TR/BR/BL slots).
//! - The Mongolian-standard preset's bubble grid stamped as empty circles.
//! - Optionally, a known answer pattern (filled discs at specified bubble centres).
//!
//! Then optionally distorts the result to mimic an MFP scan, a phone photo, or a
//! deliberately bad capture. Returns PNG bytes.

#![allow(dead_code)]

use std::path::Path;

use opencv::core::{Mat, Point, Point2f, Rect, Scalar, Size, Vector};
use opencv::{core, imgcodecs, imgproc, prelude::*};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Canonical OpenCV `DICT_6X6_50` 6×6 inner bit patterns for IDs 0..3 (1 = white,
/// 0 = black). Mirrors the constant in `shalgalt-pdf::layout::markers`. Kept local
/// rather than re-exported so the test crate does not depend on pdf internals.
const ARUCO_6X6_50: [[[u8; 6]; 6]; 4] = [
    [
        [0, 0, 0, 1, 1, 1],
        [1, 0, 0, 0, 1, 1],
        [1, 1, 0, 1, 1, 1],
        [0, 1, 1, 0, 0, 0],
        [0, 0, 1, 0, 1, 0],
        [1, 0, 0, 1, 1, 0],
    ],
    [
        [0, 0, 0, 0, 1, 1],
        [1, 0, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1, 1],
        [1, 0, 0, 0, 1, 0],
        [0, 1, 0, 0, 0, 1],
    ],
    [
        [0, 0, 0, 1, 0, 1],
        [0, 1, 1, 0, 0, 1],
        [0, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 0],
        [1, 0, 1, 0, 1, 1],
        [0, 0, 1, 1, 0, 1],
    ],
    [
        [1, 1, 0, 0, 1, 0],
        [0, 1, 0, 0, 0, 1],
        [1, 0, 1, 1, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [0, 1, 1, 0, 1, 0],
        [0, 1, 1, 1, 1, 0],
    ],
];

/// A4 portrait at 200 DPI: 1654 × 2339 px.
pub const PAGE_W: i32 = 1654;
pub const PAGE_H: i32 = 2339;

/// Render a clean MFP-style page with the given filled bubble positions
/// (normalized `[0, 1]`). Adds light gaussian noise but no rotation or distortion.
pub fn mfp_quality(filled_bubbles: &[(f32, f32)], seed: u64) -> Mat {
    let mut img = blank_page();
    draw_corner_markers(&mut img);
    draw_bubble_grid(&mut img);
    fill_bubbles(&mut img, filled_bubbles);
    add_gaussian_noise(&mut img, 4.0, seed);
    img
}

/// Render a phone-quality page: small rotation, mild perspective distortion,
/// gaussian blur, and uneven illumination.
pub fn phone_quality(filled_bubbles: &[(f32, f32)], seed: u64) -> Mat {
    let mut img = mfp_quality(filled_bubbles, seed);
    img = apply_perspective_jitter(&img, seed.wrapping_add(1));
    img = apply_blur(&img, 3);
    img = apply_uneven_illumination(&img, seed.wrapping_add(2));
    img
}

/// Heavy distortion that should make detection fail rather than crash.
pub fn deliberately_bad(seed: u64) -> Mat {
    let mut img = blank_page();
    draw_corner_markers(&mut img);
    draw_bubble_grid(&mut img);
    // Heavy noise and a partial blackout near one marker.
    add_gaussian_noise(&mut img, 60.0, seed);
    let blackout = Rect::new(0, 0, 400, 400);
    let mut roi = Mat::roi_mut(&mut img, blackout).expect("roi_mut for blackout");
    roi.set_to(&Scalar::all(0.0), &core::no_array())
        .expect("set_to black");
    img
}

/// Save `mat` as PNG at `path`.
pub fn save_png(mat: &Mat, path: &Path) {
    imgcodecs::imwrite(path.to_str().expect("utf-8 path"), mat, &Vector::new()).expect("imwrite");
}

// --- internals ------------------------------------------------------------

fn blank_page() -> Mat {
    Mat::new_rows_cols_with_default(PAGE_H, PAGE_W, core::CV_8UC1, Scalar::all(255.0))
        .expect("blank page alloc")
}

/// Marker centres in normalized space match the Mongolian-standard preset
/// (0.04 / 0.96 corners). Marker side length is 0.04 of the page.
fn draw_corner_markers(img: &mut Mat) {
    let centres = [
        (0.04_f32, 0.04_f32, 0_usize), // TL — ID 0
        (0.96, 0.04, 1),               // TR — ID 1
        (0.96, 0.96, 2),               // BR — ID 2
        (0.04, 0.96, 3),               // BL — ID 3
    ];
    let side_norm = 0.04_f32;
    for (cx_norm, cy_norm, id) in centres {
        let cx = (cx_norm * PAGE_W as f32) as i32;
        let cy = (cy_norm * PAGE_H as f32) as i32;
        let half = (side_norm * PAGE_W as f32 * 0.5) as i32;
        draw_aruco_6x6(img, cx, cy, half, id);
    }
}

/// Paint a single ArUco DICT_6X6_50 marker with 1-cell border + 6×6 inner data.
fn draw_aruco_6x6(img: &mut Mat, cx: i32, cy: i32, half: i32, id: usize) {
    // 1) Black square (border + filled interior).
    let outer = Rect::new(cx - half, cy - half, half * 2, half * 2);
    imgproc::rectangle(
        img,
        outer,
        Scalar::all(0.0),
        -1, // filled
        imgproc::LINE_8,
        0,
    )
    .expect("rectangle outer");

    // 2) For each white cell of the 6×6 inner grid, paint a white square one cell
    //    inside the border.
    let cell = (half * 2) / 8;
    let pattern = &ARUCO_6X6_50[id.min(3)];
    for (row, bits) in pattern.iter().enumerate() {
        for (col, &bit) in bits.iter().enumerate() {
            if bit == 1 {
                let x0 = cx - half + (col as i32 + 1) * cell;
                let y0 = cy - half + (row as i32 + 1) * cell;
                let cell_rect = Rect::new(x0, y0, cell, cell);
                imgproc::rectangle(img, cell_rect, Scalar::all(255.0), -1, imgproc::LINE_8, 0)
                    .expect("rectangle cell");
            }
        }
    }
}

/// Stamp an empty-circle "bubble grid" approximating the Mongolian-standard preset.
/// This is rough — a rectangular grid of 5-bubble rows. Real preset has 107 groups;
/// we draw a coarse stand-in so the warp produces plausible content.
fn draw_bubble_grid(img: &mut Mat) {
    let rows = 30;
    let cols_per_row = 5;
    let row_y_start = 0.27_f32;
    let row_y_end = 0.95_f32;
    let bubble_x_start = 0.07_f32;
    let bubble_x_step = 0.032_f32;
    let radius_norm = 0.012_f32;
    let radius_px = (radius_norm * PAGE_W as f32) as i32;

    for r in 0..rows {
        let y_norm = row_y_start + (row_y_end - row_y_start) * (r as f32 / rows.max(1) as f32);
        let y_px = (y_norm * PAGE_H as f32) as i32;
        for c in 0..cols_per_row {
            let x_norm = bubble_x_start + bubble_x_step * c as f32;
            let x_px = (x_norm * PAGE_W as f32) as i32;
            imgproc::circle(
                img,
                Point::new(x_px, y_px),
                radius_px,
                Scalar::all(0.0),
                2, // outline thickness
                imgproc::LINE_AA,
                0,
            )
            .expect("circle outline");
        }
    }
}

fn fill_bubbles(img: &mut Mat, positions: &[(f32, f32)]) {
    // Filled bubbles use a slightly larger radius than the empty-circle ring so a
    // student's filled answer covers the whole circle and a bit beyond — that is
    // closer to how a real filled-in bubble looks. LINE_8 (no antialias) keeps the
    // edge pixels strictly black for a deterministic fill measurement.
    let radius_norm = 0.014_f32;
    let radius_px = (radius_norm * PAGE_W as f32) as i32;
    for &(x_norm, y_norm) in positions {
        let cx = (x_norm * PAGE_W as f32) as i32;
        let cy = (y_norm * PAGE_H as f32) as i32;
        imgproc::circle(
            img,
            Point::new(cx, cy),
            radius_px,
            Scalar::all(0.0),
            -1, // filled
            imgproc::LINE_8,
            0,
        )
        .expect("filled circle");
    }
}

fn add_gaussian_noise(img: &mut Mat, sigma: f64, seed: u64) {
    // Generate signed noise in 16S (mean = 0), add to the 8U image after promoting it
    // to 16S, then clamp back to 8U. Doing the math directly in 8U with a mean-128
    // bias trick collapses the dynamic range: white pixels (255) saturate at the
    // upper bound when 128 is added, ink pixels (0) clamp at +0, and the resulting
    // bias subtraction leaves paper at 127 instead of 255. `core::randn` itself runs
    // in C and is two orders of magnitude faster than a per-pixel Rust loop in debug
    // builds.
    core::set_rng_seed(seed as i32).expect("set_rng_seed");
    let mut noise =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), core::CV_16SC1, Scalar::all(0.0))
            .expect("noise alloc");
    core::randn(&mut noise, &Scalar::all(0.0), &Scalar::all(sigma)).expect("randn");

    let mut img_16s = Mat::default();
    img.convert_to(&mut img_16s, core::CV_16SC1, 1.0, 0.0)
        .expect("convert to 16s");
    let mut summed = Mat::default();
    core::add(
        &img_16s,
        &noise,
        &mut summed,
        &core::no_array(),
        core::CV_16SC1,
    )
    .expect("add noise");
    let mut clamped = Mat::default();
    summed
        .convert_to(&mut clamped, core::CV_8UC1, 1.0, 0.0)
        .expect("convert back to 8u");
    *img = clamped;
}

fn apply_perspective_jitter(src: &Mat, seed: u64) -> Mat {
    let mut rng = StdRng::seed_from_u64(seed);
    let w = src.cols() as f32;
    let h = src.rows() as f32;
    let mut jitter = |max: f32| -> f32 { rng.gen_range(-max..=max) };
    let jx = w * 0.015;
    let jy = h * 0.015;
    let src_pts = [
        Point2f::new(0.0, 0.0),
        Point2f::new(w, 0.0),
        Point2f::new(w, h),
        Point2f::new(0.0, h),
    ];
    let dst_pts = [
        Point2f::new(jitter(jx), jitter(jy)),
        Point2f::new(w + jitter(jx), jitter(jy)),
        Point2f::new(w + jitter(jx), h + jitter(jy)),
        Point2f::new(jitter(jx), h + jitter(jy)),
    ];
    let m = imgproc::get_perspective_transform_slice_def(&src_pts, &dst_pts)
        .expect("get_perspective_transform");
    let mut dst = Mat::default();
    imgproc::warp_perspective(
        src,
        &mut dst,
        &m,
        Size::new(src.cols(), src.rows()),
        imgproc::INTER_LINEAR,
        core::BORDER_CONSTANT,
        Scalar::all(255.0),
    )
    .expect("warp_perspective synth");
    dst
}

fn apply_blur(src: &Mat, ksize: i32) -> Mat {
    let mut dst = Mat::default();
    imgproc::gaussian_blur(
        src,
        &mut dst,
        Size::new(ksize, ksize),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )
    .expect("gaussian_blur");
    dst
}

fn apply_uneven_illumination(src: &Mat, seed: u64) -> Mat {
    let mut rng = StdRng::seed_from_u64(seed);
    let amp: f64 = rng.gen_range(20.0..40.0);
    let cols = src.cols();
    let rows = src.rows();

    // Build a vignette darkening map: 0 at the centre, `amp` at the corners. Done
    // via a Gaussian-blurred radial mask rather than a per-pixel loop so debug-mode
    // fixture generation stays under a second.
    let mut radial = Mat::new_rows_cols_with_default(rows, cols, core::CV_8UC1, Scalar::all(0.0))
        .expect("radial alloc");
    imgproc::circle(
        &mut radial,
        Point::new(cols / 2, rows / 2),
        (cols.min(rows)) / 8,
        Scalar::all(255.0),
        -1,
        imgproc::LINE_8,
        0,
    )
    .expect("circle radial");
    // Heavy blur turns the disc into a smooth bright spot at the centre, dark in
    // corners — the inverse of what we want, so we invert and scale to `amp`.
    let mut blurred = Mat::default();
    // 99-pixel kernel is wide enough to give a smooth radial falloff at A4 200 DPI
    // and avoids the multi-second cost of `gaussian_blur` with a sigma derived from
    // the page width (which forces an enormous kernel).
    imgproc::gaussian_blur(
        &radial,
        &mut blurred,
        Size::new(99, 99),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )
    .expect("blur radial");

    let mut inverted = Mat::default();
    core::bitwise_not(&blurred, &mut inverted, &core::no_array()).expect("invert radial");

    let mut scaled = Mat::default();
    inverted
        .convert_to(&mut scaled, core::CV_8U, amp / 255.0, 0.0)
        .expect("scale radial");

    let mut dst = Mat::default();
    core::subtract(src, &scaled, &mut dst, &core::no_array(), -1).expect("subtract vignette");
    dst
}
