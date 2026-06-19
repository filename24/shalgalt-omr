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
use opencv::{calib3d, core, imgproc, prelude::*};

use shalgalt_core::error::{AppError, AppResult};

/// Output canvas size, in pixels, after perspective warp. Picked to match a 200-DPI
/// rasterization of A4 portrait (~1654×2339 px) but rounded for stability.
pub const WARP_OUT_W: i32 = 1700;
pub const WARP_OUT_H: i32 = 2400;

/// One detected ArUco marker.
#[derive(Debug, Clone)]
pub struct DetectedMarker {
    pub id: i32,
    /// The marker's four corners in source-image pixel coordinates, in the order
    /// ArUco returns them: clockwise from the marker's own top-left — `[TL, TR, BR, BL]`.
    /// Keeping all four (instead of averaging to a centre) lets a homography be solved
    /// from as few as three markers — 12 correspondences — so one occluded corner no
    /// longer fails the whole page.
    pub corners: [Point2f; 4],
}

/// Detect ArUco DICT_6X6_50 markers in `gray` and return the detected subset of
/// IDs 0..3, sorted by id. Returns `BadRequest` if fewer than [`MIN_MARKERS`] are
/// found — three markers (12 corners spanning three page corners) are enough for a
/// well-conditioned homography.
pub fn detect_corner_markers(gray: &Mat) -> AppResult<Vec<DetectedMarker>> {
    let (corners, ids) = detect_raw(gray)?;
    assemble_corner_set(&corners, &ids)
}

/// Minimum number of the four corner markers that must be detected to align a page.
/// Below this the homography is too poorly constrained to trust.
pub const MIN_MARKERS: usize = 3;

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

/// Assemble the `(corners, ids)` output of either ArUco backend into the detected
/// subset of corner markers (IDs 0..3), each carrying its four corners, sorted by id.
/// Fails only when fewer than [`MIN_MARKERS`] are present — a partial set still aligns.
fn assemble_corner_set(corners: &Vector<Mat>, ids: &Mat) -> AppResult<Vec<DetectedMarker>> {
    let count = ids.rows();

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
        // Each corner Mat is 1×4, channel-2 (Point2f), clockwise from the marker's
        // top-left. Keep all four — they become the homography correspondences.
        let mut marker_corners = [Point2f::new(0.0, 0.0); 4];
        for (c, slot) in marker_corners.iter_mut().enumerate() {
            *slot = *pts
                .at_2d::<core::Point2f>(0, c as i32)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("aruco: corner[{c}]: {e}")))?;
        }
        found[slot] = Some(DetectedMarker {
            id: raw_id,
            corners: marker_corners,
        });
    }

    // Report present/missing once so a partial or failed alignment names exactly
    // which corner(s) the detector lost.
    let present: Vec<usize> = found
        .iter()
        .enumerate()
        .filter(|(_, m)| m.is_some())
        .map(|(slot, _)| slot)
        .collect();
    if present.len() < MIN_MARKERS {
        let missing: Vec<usize> = found
            .iter()
            .enumerate()
            .filter(|(_, m)| m.is_none())
            .map(|(slot, _)| slot)
            .collect();
        tracing::warn!(
            ?present,
            ?missing,
            "aruco: too few markers detected; cannot align page"
        );
        return Err(AppError::BadRequest(format!(
            "ArUco markers insufficient: found IDs {present:?}, missing {missing:?} \
             (need at least {MIN_MARKERS} of 4 — DICT_6X6_50). Ensure the corner markers \
             are visible, unobscured, and not cropped."
        )));
    }

    Ok(found.into_iter().flatten().collect())
}

/// Fit a perspective transform from the detected markers' corners to their canonical
/// positions on a `WARP_OUT_W × WARP_OUT_H` canvas, then warp `src` through it.
///
/// Each detected marker contributes four correspondences (its corners ↔ the canonical
/// corner destinations in [`MarkerLayout`]), so three markers already over-determine the
/// homography. We use [`calib3d::find_homography`] with RANSAC: it least-squares-fits the
/// 12–16 points and rejects a single mis-located corner. The Mongolian-standard preset
/// places marker centres at (0.04, 0.04), (0.96, 0.04), (0.96, 0.96), (0.04, 0.96).
pub fn warp_to_canonical(
    src: &Mat,
    markers: &[DetectedMarker],
    layout: &MarkerLayout,
) -> AppResult<Mat> {
    let mut src_pts: Vector<Point2f> = Vector::new();
    let mut dst_pts: Vector<Point2f> = Vector::new();
    for marker in markers {
        let dst = layout.corners_for(marker.id).ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "aruco: marker id {} out of range 0..3",
                marker.id
            ))
        })?;
        for (s, d) in marker.corners.iter().zip(dst.iter()) {
            src_pts.push(*s);
            dst_pts.push(*d);
        }
    }

    let m = calib3d::find_homography(
        &src_pts,
        &dst_pts,
        &mut Mat::default(),
        calib3d::RANSAC,
        3.0,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("find_homography: {e}")))?;
    if m.empty() {
        return Err(AppError::BadRequest(
            "could not fit a perspective transform from the detected markers".into(),
        ));
    }

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

/// Canonical-pixel destinations for the perspective warp. For each marker id (0..3),
/// the positions of its four corners on the `WARP_OUT_W × WARP_OUT_H` canvas, in
/// `[TL, TR, BR, BL]` order — the same order ArUco reports detected corners, so
/// correspondences line up by index.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLayout {
    corners: [[Point2f; 4]; 4],
}

impl MarkerLayout {
    /// Build a `MarkerLayout` from the template's `[Marker; 4]` array. Markers are
    /// stored in TL/TR/BR/BL order with ArUco ids 0..3 (master plan §6.5).
    ///
    /// The marker is a square in millimetres; the canonical canvas aspect
    /// (`WARP_OUT_W:WARP_OUT_H` ≈ A4) is chosen to match the print paper, so a single
    /// square half-extent in canvas pixels reproduces the printed corner geometry to
    /// within a fraction of a pixel — no `PaperSpec` is needed here.
    pub fn from_template(markers: &[shalgalt_core::domain::Marker; 4]) -> Self {
        let corners_of = |m: &shalgalt_core::domain::Marker| {
            let cx = m.position.x * WARP_OUT_W as f32;
            let cy = m.position.y * WARP_OUT_H as f32;
            let half = m.size * 0.5 * WARP_OUT_W.min(WARP_OUT_H) as f32;
            // Template y grows downward, matching ArUco's [TL, TR, BR, BL] corner order.
            [
                Point2f::new(cx - half, cy - half), // TL
                Point2f::new(cx + half, cy - half), // TR
                Point2f::new(cx + half, cy + half), // BR
                Point2f::new(cx - half, cy + half), // BL
            ]
        };
        Self {
            corners: [
                corners_of(&markers[0]),
                corners_of(&markers[1]),
                corners_of(&markers[2]),
                corners_of(&markers[3]),
            ],
        }
    }

    /// The four canonical-pixel corner destinations for marker `id` (0..3), or `None`
    /// if `id` is outside that range.
    fn corners_for(&self, id: i32) -> Option<&[Point2f; 4]> {
        usize::try_from(id).ok().and_then(|i| self.corners.get(i))
    }
}
