//! Marker-based perspective alignment.
//!
//! Detect the four ArUco DICT_6X6_50 markers (IDs 0..3 in TL/TR/BR/BL order — see
//! `docs/adr/0009-aruco-markers.md`), then compute the homography that maps each
//! marker centre back to its expected position in the template's normalized coordinate
//! space. The resulting warped image is the canvas that `bubbles` reads.
//!
//! Failure to detect all four markers is **not** an internal error — it is the most
//! common reason a hand-photographed sheet fails grading. Callers translate
//! `AppError::BadRequest("markers not found")` into a Mongolian "хуудас танигдсангүй"
//! toast for the user.
//!
//! ## OpenCV version split
//!
//! OpenCV 4.7 moved ArUco from the contrib `cv::aruco` module into `cv::objdetect`
//! and replaced the free function `detect_markers` with the `ArucoDetector` class.
//! Ubuntu apt still ships 4.6; the Windows self-extractor and current Homebrew
//! ship 4.8+. The two implementations below are selected at build time by `build.rs`
//! via `opencv_aruco_legacy` / `opencv_aruco_modern` cfg flags.

use opencv::core::{Mat, Point2f, Size, Vector};
use opencv::{core, imgproc, prelude::*};

use shalgalt_core::error::{AppError, AppResult};

/// Output canvas size, in pixels, after perspective warp. Picked to match a 200-DPI
/// rasterization of A4 portrait (~1654×2339 px) but rounded for stability.
pub const WARP_OUT_W: i32 = 1700;
pub const WARP_OUT_H: i32 = 2400;

/// One detected ArUco marker.
#[derive(Debug, Clone)]
pub struct DetectedMarker {
    pub id: i32,
    /// Centre of the marker in source-image pixel coordinates.
    pub centre: Point2f,
}

/// Detect ArUco DICT_6X6_50 markers in `gray` and return a `[TL, TR, BR, BL]`
/// quadruple. Returns `BadRequest` if any of IDs 0..3 is missing.
pub fn detect_corner_markers(gray: &Mat) -> AppResult<[DetectedMarker; 4]> {
    let (corners, ids) = detect_raw(gray)?;
    assemble_corner_set(&corners, &ids)
}

#[cfg(opencv_aruco_legacy)]
fn detect_raw(gray: &Mat) -> AppResult<(Vector<Mat>, Mat)> {
    use opencv::aruco::{
        self, get_predefined_dictionary_i32, DetectorParameters, Dictionary, DICT_6X6_50,
    };
    use opencv::core::Ptr;

    let dictionary_ptr: Ptr<Dictionary> = get_predefined_dictionary_i32(DICT_6X6_50)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: dictionary: {e}")))?;
    let params_ptr: Ptr<DetectorParameters> = DetectorParameters::create()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: params: {e}")))?;

    let mut corners: Vector<Mat> = Vector::new();
    let mut ids: Mat = Mat::default();
    let mut rejected: Vector<Mat> = Vector::new();
    #[allow(deprecated)]
    aruco::detect_markers(
        gray,
        &dictionary_ptr,
        &mut corners,
        &mut ids,
        &params_ptr,
        &mut rejected,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: detect: {e}")))?;
    Ok((corners, ids))
}

#[cfg(opencv_aruco_modern)]
fn detect_raw(gray: &Mat) -> AppResult<(Vector<Mat>, Mat)> {
    use opencv::objdetect::{
        get_predefined_dictionary, ArucoDetector, DetectorParameters, PredefinedDictionaryType,
        RefineParameters,
    };

    let dictionary = get_predefined_dictionary(PredefinedDictionaryType::DICT_6X6_50)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: dictionary: {e}")))?;
    let params = DetectorParameters::default()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: params: {e}")))?;
    let refine = RefineParameters::new(10.0, 3.0, true)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: refine params: {e}")))?;
    let detector = ArucoDetector::new(&dictionary, &params, refine)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: detector: {e}")))?;

    let mut corners: Vector<Mat> = Vector::new();
    let mut ids: Mat = Mat::default();
    let mut rejected: Vector<Mat> = Vector::new();
    detector
        .detect_markers(gray, &mut corners, &mut ids, &mut rejected)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: detect: {e}")))?;
    Ok((corners, ids))
}

/// Assemble the `(corners, ids)` output of either ArUco backend into the
/// `[TL, TR, BR, BL]` ordering the rest of the pipeline expects.
fn assemble_corner_set(corners: &Vector<Mat>, ids: &Mat) -> AppResult<[DetectedMarker; 4]> {
    let count = ids.rows();
    if count < 4 {
        return Err(AppError::BadRequest(format!(
            "expected 4 ArUco markers (DICT_6X6_50, IDs 0..3), found {count}"
        )));
    }

    let mut found: [Option<DetectedMarker>; 4] = [None, None, None, None];
    for i in 0..count {
        let raw_id = *ids
            .at_2d::<i32>(i, 0)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: ids[{i}]: {e}")))?;
        if !(0..=3).contains(&raw_id) {
            // Marker not in our set — ignore. Real scans may pick up stray ArUco-like
            // patterns from page noise.
            continue;
        }
        let slot = raw_id as usize;
        if found[slot].is_some() {
            continue;
        }
        let pts = corners
            .get(i as usize)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: corners[{i}]: {e}")))?;
        // Each corner Mat is 1×4, channel-2 (Point2f). Sum the four corners and divide
        // by 4 to get the marker centre.
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        for c in 0..4 {
            let p = pts
                .at_2d::<core::Point2f>(0, c)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: corner[{c}]: {e}")))?;
            sum_x += p.x;
            sum_y += p.y;
        }
        found[slot] = Some(DetectedMarker {
            id: raw_id,
            centre: Point2f::new(sum_x / 4.0, sum_y / 4.0),
        });
    }

    let mut out: Vec<DetectedMarker> = Vec::with_capacity(4);
    for (slot, m) in found.into_iter().enumerate() {
        match m {
            Some(m) => out.push(m),
            None => {
                return Err(AppError::BadRequest(format!(
                    "ArUco marker with ID {slot} not found"
                )));
            }
        }
    }
    Ok([out.remove(0), out.remove(0), out.remove(0), out.remove(0)])
}

/// Compute the perspective transform that maps the four detected marker centres to
/// the four corners of a canonical `WARP_OUT_W × WARP_OUT_H` canvas, then warp `src`
/// through it.
///
/// Marker positions in normalized template coordinates are read from
/// [`MarkerLayout`]. The Mongolian-standard preset places markers at (0.04, 0.04),
/// (0.96, 0.04), (0.96, 0.96), (0.04, 0.96).
pub fn warp_to_canonical(
    src: &Mat,
    markers: &[DetectedMarker; 4],
    layout: &MarkerLayout,
) -> AppResult<Mat> {
    let src_pts = [
        markers[0].centre,
        markers[1].centre,
        markers[2].centre,
        markers[3].centre,
    ];
    let dst_pts = [
        Point2f::new(
            layout.tl.0 * WARP_OUT_W as f32,
            layout.tl.1 * WARP_OUT_H as f32,
        ),
        Point2f::new(
            layout.tr.0 * WARP_OUT_W as f32,
            layout.tr.1 * WARP_OUT_H as f32,
        ),
        Point2f::new(
            layout.br.0 * WARP_OUT_W as f32,
            layout.br.1 * WARP_OUT_H as f32,
        ),
        Point2f::new(
            layout.bl.0 * WARP_OUT_W as f32,
            layout.bl.1 * WARP_OUT_H as f32,
        ),
    ];

    let m = imgproc::get_perspective_transform_slice_def(&src_pts, &dst_pts)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("get_perspective_transform: {e}")))?;

    let mut warped = Mat::default();
    imgproc::warp_perspective(
        src,
        &mut warped,
        &m,
        Size::new(WARP_OUT_W, WARP_OUT_H),
        imgproc::INTER_LINEAR,
        core::BORDER_CONSTANT,
        core::Scalar::all(255.0),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("warp_perspective: {e}")))?;

    Ok(warped)
}

/// Normalized marker positions read from the template. Used as the destination
/// quadrilateral when we warp the scanned page back to canonical space.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLayout {
    pub tl: (f32, f32),
    pub tr: (f32, f32),
    pub br: (f32, f32),
    pub bl: (f32, f32),
}

impl MarkerLayout {
    /// Build a `MarkerLayout` from the template's `[Marker; 4]` array. Markers are
    /// stored in TL/TR/BR/BL order (master plan §6.5).
    pub fn from_template(markers: &[shalgalt_core::domain::Marker; 4]) -> Self {
        Self {
            tl: (markers[0].position.x, markers[0].position.y),
            tr: (markers[1].position.x, markers[1].position.y),
            br: (markers[2].position.x, markers[2].position.y),
            bl: (markers[3].position.x, markers[3].position.y),
        }
    }
}
