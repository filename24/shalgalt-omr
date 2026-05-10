//! Detect the local OpenCV version so the ArUco bindings target the right module.
//!
//! OpenCV 4.7 moved the ArUco contrib API into `cv::objdetect` and renamed it
//! to `ArucoDetector`. Ubuntu / Debian apt and the older Homebrew bottles still
//! ship 4.6, but the Windows self-extractor and current Homebrew ship 4.8+.
//! opencv-rust 0.94 surfaces whatever the underlying headers expose — so on
//! 4.6 the `cv::aruco` module exists and on 4.8+ it does not.
//!
//! This script emits one of two cfg flags so `src/perspective.rs` can pick the
//! right import path:
//!
//! - `opencv_aruco_legacy` ⇒ `use opencv::aruco::*;`           (OpenCV 4.6)
//! - `opencv_aruco_modern` ⇒ `use opencv::objdetect::*;`       (OpenCV 4.7+)
//!
//! Detection order:
//! 1. `OPENCV_FORCE_ARUCO=legacy|modern` env var override.
//! 2. `pkg-config --modversion opencv4` (Linux / macOS).
//! 3. `OPENCV_VERSION` env var (some Windows / vcpkg setups).
//! 4. Fallback: assume modern (4.7+). Windows users without pkg-config land here.

use std::process::Command;

fn parse_version(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    let mut parts = s.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    Some((major, minor))
}

fn detect() -> (u32, u32) {
    if let Ok(force) = std::env::var("OPENCV_FORCE_ARUCO") {
        match force.as_str() {
            "legacy" => return (4, 6),
            "modern" => return (4, 8),
            other => {
                println!(
                    "cargo:warning=OPENCV_FORCE_ARUCO=\"{other}\" not recognized; \
                     expected \"legacy\" or \"modern\". Falling back to auto-detect."
                );
            }
        }
    }

    if let Ok(out) = Command::new("pkg-config")
        .args(["--modversion", "opencv4"])
        .output()
    {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if let Some(v) = parse_version(&s) {
                    return v;
                }
            }
        }
    }

    if let Ok(s) = std::env::var("OPENCV_VERSION") {
        if let Some(v) = parse_version(&s) {
            return v;
        }
    }

    // Windows self-extractor + current vcpkg ship 4.8+ as of 2026-05; default modern.
    (4, 8)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OPENCV_FORCE_ARUCO");
    println!("cargo:rerun-if-env-changed=OPENCV_VERSION");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    // Allow our two cfg flags so rustc 1.80+ doesn't warn about unknown cfgs.
    println!("cargo:rustc-check-cfg=cfg(opencv_aruco_legacy)");
    println!("cargo:rustc-check-cfg=cfg(opencv_aruco_modern)");

    let (major, minor) = detect();
    if major == 4 && minor < 7 {
        println!("cargo:rustc-cfg=opencv_aruco_legacy");
        println!("cargo:warning=shalgalt-cv: detected OpenCV {major}.{minor} — using legacy cv::aruco bindings");
    } else {
        println!("cargo:rustc-cfg=opencv_aruco_modern");
        println!(
            "cargo:warning=shalgalt-cv: detected OpenCV {major}.{minor} — using cv::objdetect::ArucoDetector"
        );
    }
}
