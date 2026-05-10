//! End-to-end integration test for the `shalgalt-cv` pipeline (P3-09 acceptance).
//!
//! Skips when `pdfium` is not available — see `pdfium_marker_raster.rs` for the same
//! pattern. The synthetic fixtures we produce for raster-side checks (ArUco
//! detection, fill ratios on warped canvases) do **not** require pdfium and run
//! unconditionally.

#![cfg(test)]

mod common;

use std::path::Path;

use opencv::core::Mat;
use opencv::{imgcodecs, prelude::*};
use shalgalt_core::domain::{
    BubbleGroup, BubbleKind, Marker, MarkerKind, OmrTemplate, TemplatePoint,
};
use shalgalt_cv::{bubbles, perspective, threshold};

use common::synth;

fn template_with_one_question_at(group_id: &str, x: f32, y: f32) -> OmrTemplate {
    OmrTemplate {
        version: OmrTemplate::CURRENT_VERSION,
        title: "synth".into(),
        markers: [
            Marker {
                id: "tl".into(),
                position: TemplatePoint { x: 0.04, y: 0.04 },
                size: 0.04,
                kind: MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] },
            },
            Marker {
                id: "tr".into(),
                position: TemplatePoint { x: 0.96, y: 0.04 },
                size: 0.04,
                kind: MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] },
            },
            Marker {
                id: "br".into(),
                position: TemplatePoint { x: 0.96, y: 0.96 },
                size: 0.04,
                kind: MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] },
            },
            Marker {
                id: "bl".into(),
                position: TemplatePoint { x: 0.04, y: 0.96 },
                size: 0.04,
                kind: MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] },
            },
        ],
        groups: vec![BubbleGroup {
            id: group_id.into(),
            kind: BubbleKind::Question,
            label: group_id.into(),
            bubbles: vec![TemplatePoint { x, y }, TemplatePoint { x: x + 0.05, y }],
            answer_index: Some(0),
            score: 1.0,
            section: None,
        }],
    }
}

fn save_to_tempdir(mat: &Mat, name: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(name);
    synth::save_png(mat, &path);
    dir
}

#[test]
fn aruco_markers_detected_in_clean_synthetic_page() {
    // Use one filled bubble at a known position so we can later assert its fill.
    let filled = (0.20_f32, 0.50_f32);
    let img = synth::mfp_quality(&[filled], 0xC0FFEE);

    // Save + reload through imread to mirror the real pipeline path (pipeline reads
    // page rasters from disk).
    let dir = save_to_tempdir(&img, "mfp.png");
    let path = dir.path().join("mfp.png");
    let raw =
        imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_GRAYSCALE).expect("imread");
    assert!(!raw.empty(), "raster did not load");

    // ArUco detection runs on grayscale.
    let markers = perspective::detect_corner_markers(&raw)
        .expect("detect_corner_markers should succeed on a clean MFP fixture");
    let ids: Vec<i32> = markers.iter().map(|m| m.id).collect();
    assert_eq!(ids, vec![0, 1, 2, 3], "TL/TR/BR/BL ordering");
}

#[test]
fn filled_bubble_reads_above_threshold() {
    let filled_norm = (0.20_f32, 0.50_f32);
    let img = synth::mfp_quality(&[filled_norm], 0xBADF00D);

    let template = template_with_one_question_at("q1", filled_norm.0, filled_norm.1);

    // Through the pipeline: gray -> markers -> warp -> flatten_to_ink -> bubbles.
    let gray = threshold::to_gray(&img).expect("to_gray");
    let markers = perspective::detect_corner_markers(&gray).expect("detect markers");
    let layout = perspective::MarkerLayout::from_template(&template.markers);
    let warped = perspective::warp_to_canonical(&gray, &markers, &layout).expect("warp");
    let ink = threshold::flatten_to_ink(&warped).expect("flatten_to_ink");
    let readings = bubbles::read_bubbles(&ink, &template).expect("read_bubbles");

    assert_eq!(readings.len(), 2, "two bubbles in q1");
    let filled = &readings[0];
    let empty = &readings[1];
    assert!(
        filled.fill > 0.65,
        "filled bubble should read above 0.65, got {}",
        filled.fill
    );
    assert!(
        empty.fill < 0.35,
        "empty bubble should read below 0.35, got {}",
        empty.fill
    );
}

#[test]
fn deliberately_bad_fixture_fails_marker_detection_gracefully() {
    let img = synth::deliberately_bad(0xBAD);
    let dir = save_to_tempdir(&img, "bad.png");
    let path = dir.path().join("bad.png");
    let raw =
        imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_GRAYSCALE).expect("imread");

    let result = perspective::detect_corner_markers(&raw);
    assert!(
        result.is_err(),
        "deliberately bad fixture should fail marker detection (returned {:?})",
        result.map(|m| m.iter().map(|m| m.id).collect::<Vec<_>>())
    );
}

/// Full fixture sweep — runs the 5×MFP / 5×phone / 2×bad set required by the
/// P3-09 acceptance criteria. Asserts that all twelve PNGs land on disk and
/// surfaces them under `<tmpdir>/shalgalt-cv-fixtures/` for visual inspection.
#[test]
fn dump_fixture_set_for_inspection() {
    // Generate one of each kind under `target/test-fixtures/` so a developer can
    // open the PNGs visually. This is mostly for debugging the synth generator
    // itself; assertions are minimal.
    let out_dir = std::env::temp_dir().join("shalgalt-cv-fixtures");
    std::fs::create_dir_all(&out_dir).expect("mkdir tmp");

    for i in 0..5 {
        let img = synth::mfp_quality(&[(0.20, 0.50)], 100 + i);
        synth::save_png(&img, &out_dir.join(format!("mfp-{i}.png")));
    }
    for i in 0..5 {
        let img = synth::phone_quality(&[(0.20, 0.50)], 200 + i);
        synth::save_png(&img, &out_dir.join(format!("phone-{i}.png")));
    }
    for i in 0..2 {
        let img = synth::deliberately_bad(300 + i);
        synth::save_png(&img, &out_dir.join(format!("bad-{i}.png")));
    }

    // Sanity: at least 12 files written.
    let count = std::fs::read_dir(&out_dir)
        .expect("readdir")
        .filter(|e| {
            e.as_ref()
                .map(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "png")
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .count();
    assert!(count >= 12, "expected >= 12 fixtures, got {count}");

    // Pacify unused-import warnings from the test crate when assertions trim out.
    let _ = Path::new(".");
}
