//! P2-06 acceptance test — rasterize a `shalgalt-pdf`-generated OMR PDF via
//! `pdfium-render` and verify that the four corner markers land at the expected pixel
//! positions (±2 mm tolerance).
//!
//! The test only carries weight where `pdfium-render` can dynamically bind to libpdfium.
//! When the library is missing (e.g. some CI runners) we **skip** instead of failing so
//! `cargo test --workspace` stays green. Set `OMR_REQUIRE_PDFIUM=1` to force a hard
//! failure in environments where pdfium must be present.

use std::path::Path;

use image::GenericImageView;
use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

use shalgalt_pdf::{
    domain::{Marker, MarkerKind, OmrTemplate, TemplatePoint},
    render_template, PdfOptions,
};

const TARGET_DPI: f32 = 200.0;
const A4_WIDTH_MM: f32 = 210.0;
const A4_HEIGHT_MM: f32 = 297.0;

fn try_bind() -> Option<Pdfium> {
    let library_name = Pdfium::pdfium_platform_library_name();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join(&library_name);
            if let Ok(b) = Pdfium::bind_to_library(&candidate) {
                return Some(Pdfium::new(b));
            }
        }
    }
    Pdfium::bind_to_system_library().ok().map(Pdfium::new)
}

fn minimal_template() -> OmrTemplate {
    let marker = |id: &str, x: f32, y: f32| Marker {
        id: id.to_string(),
        position: TemplatePoint { x, y },
        size: 0.04,
        kind: MarkerKind::Square,
    };
    OmrTemplate {
        version: OmrTemplate::CURRENT_VERSION,
        title: "marker raster".into(),
        markers: [
            marker("tl", 0.05, 0.05),
            marker("tr", 0.95, 0.05),
            marker("br", 0.95, 0.95),
            marker("bl", 0.05, 0.95),
        ],
        groups: vec![],
    }
}

/// True when the `±half_px` window around `(cx_px, cy_px)` carries enough dark pixels
/// (luma < 80) to count as a printed marker.
fn region_is_dark(img: &image::DynamicImage, cx_px: u32, cy_px: u32, half_px: u32) -> bool {
    let x0 = cx_px.saturating_sub(half_px);
    let y0 = cy_px.saturating_sub(half_px);
    let x1 = (cx_px + half_px).min(img.width());
    let y1 = (cy_px + half_px).min(img.height());

    let mut dark = 0_u32;
    let mut total = 0_u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y);
            // Plain luma average.
            let luma = (p[0] as u16 + p[1] as u16 + p[2] as u16) / 3;
            if luma < 80 {
                dark += 1;
            }
            total += 1;
        }
    }
    total > 0 && (dark as f32 / total as f32) > 0.10
}

#[test]
fn corner_markers_appear_at_expected_pixel_positions() {
    let pdfium = match try_bind() {
        Some(p) => p,
        None => {
            if std::env::var_os("OMR_REQUIRE_PDFIUM").is_some() {
                panic!("OMR_REQUIRE_PDFIUM=1 set but libpdfium is not available");
            }
            eprintln!(
                "skipping: could not bind a pdfium dynamic library. The P2-06 raster check \
                 only runs when pdfium is installed."
            );
            return;
        }
    };

    // 1) Render a 4-corner-only PDF via shalgalt-pdf into memory.
    let template = minimal_template();
    let bytes = render_template(&template, &PdfOptions::default()).expect("render");

    // pdfium-render only accepts a path, so spill the bytes to a temp file.
    let dir = tempfile::tempdir().expect("tempdir");
    let pdf_path = dir.path().join("markers.pdf");
    std::fs::write(&pdf_path, &bytes).expect("write tmp pdf");

    // 2) Rasterize at 200 DPI.
    let doc = pdfium
        .load_pdf_from_file(&pdf_path, None)
        .expect("load pdf");
    let page = doc.pages().first().expect("first page");

    let target_w_px = (A4_WIDTH_MM / 25.4 * TARGET_DPI) as i32;
    let cfg = PdfRenderConfig::new()
        .set_target_width(target_w_px)
        .set_maximum_height((A4_HEIGHT_MM / 25.4 * TARGET_DPI) as i32);
    let img = page
        .render_with_config(&cfg)
        .expect("render page")
        .as_image();

    debug_dump_path(&img, &pdf_path);

    // 3) Map mm → pixel. Markers sit at normalized (0.05, 0.05) / (0.95, 0.05) /
    //    (0.95, 0.95) / (0.05, 0.95).
    let px_per_mm = img.width() as f32 / A4_WIDTH_MM;
    let half_px = (3.0 * px_per_mm) as u32; // ±3 mm sample window (markers are ~8.4 mm wide)

    let expected = [
        (0.05_f32, 0.05_f32),
        (0.95, 0.05),
        (0.95, 0.95),
        (0.05, 0.95),
    ];
    for (i, (nx, ny)) in expected.iter().enumerate() {
        let cx_px = (nx * A4_WIDTH_MM * px_per_mm) as u32;
        let cy_px = (ny * A4_HEIGHT_MM * px_per_mm) as u32;
        assert!(
            region_is_dark(&img, cx_px, cy_px, half_px),
            "marker {i} at normalized ({nx}, {ny}) → ({cx_px}, {cy_px}) px is not dark enough"
        );
    }
}

fn debug_dump_path(_img: &image::DynamicImage, _pdf_path: &Path) {
    if std::env::var_os("OMR_DUMP_RASTER").is_some() {
        let png = _pdf_path.with_extension("png");
        let _ = _img.save(&png);
        eprintln!("wrote raster to {}", png.display());
    }
}
