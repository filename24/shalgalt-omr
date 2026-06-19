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
//! 4. `OPENCV_INCLUDE_PATHS` → read `opencv2/core/version.hpp` directly. This
//!    mirrors how opencv-rust itself finds the version and catches Windows /
//!    vcpkg / custom setups where pkg-config is unavailable but the headers are.
//! 5. Fallback: assume modern (4.7+). If this guess is wrong the build fails
//!    later with unresolved `cv::objdetect` ArUco imports — the warning below
//!    points the user at `OPENCV_FORCE_ARUCO`.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

fn parse_version(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    let mut parts = s.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Read `CV_VERSION_MAJOR` / `CV_VERSION_MINOR` out of an OpenCV `version.hpp`.
fn version_from_header(version_hpp: &PathBuf) -> Option<(u32, u32)> {
    let reader = BufReader::new(File::open(version_hpp).ok()?);
    let (mut major, mut minor) = (None, None);
    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim();
        // Lines look like: `#define CV_VERSION_MAJOR    4`
        if let Some(rest) = line.strip_prefix("#define CV_VERSION_") {
            let mut parts = rest.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some("MAJOR"), Some(v)) => major = v.parse().ok(),
                (Some("MINOR"), Some(v)) => minor = v.parse().ok(),
                _ => {}
            }
        }
        if let (Some(maj), Some(min)) = (major, minor) {
            return Some((maj, min));
        }
    }
    None
}

/// Probe every directory in `OPENCV_INCLUDE_PATHS` for `opencv2/core/version.hpp`.
/// opencv-rust treats this var as a comma-separated list; a leading `+` means
/// "in addition to autodetected paths" and is stripped here.
fn version_from_include_paths() -> Option<(u32, u32)> {
    let raw = std::env::var("OPENCV_INCLUDE_PATHS").ok()?;
    let raw = raw.strip_prefix('+').unwrap_or(&raw);
    for dir in raw.split(',') {
        let dir = dir.trim();
        if dir.is_empty() {
            continue;
        }
        let candidate = PathBuf::from(dir).join("opencv2/core/version.hpp");
        if let Some(v) = version_from_header(&candidate) {
            return Some(v);
        }
    }
    None
}

/// Detect the OpenCV `(major, minor)` version, or `None` if every probe fails.
fn detect() -> Option<(u32, u32)> {
    if let Ok(force) = std::env::var("OPENCV_FORCE_ARUCO") {
        match force.as_str() {
            "legacy" => return Some((4, 6)),
            "modern" => return Some((4, 8)),
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
                    return Some(v);
                }
            }
        }
    }

    if let Ok(s) = std::env::var("OPENCV_VERSION") {
        if let Some(v) = parse_version(&s) {
            return Some(v);
        }
    }

    version_from_include_paths()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OPENCV_FORCE_ARUCO");
    println!("cargo:rerun-if-env-changed=OPENCV_VERSION");
    println!("cargo:rerun-if-env-changed=OPENCV_INCLUDE_PATHS");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    // Allow our two cfg flags so rustc 1.80+ doesn't warn about unknown cfgs.
    println!("cargo:rustc-check-cfg=cfg(opencv_aruco_legacy)");
    println!("cargo:rustc-check-cfg=cfg(opencv_aruco_modern)");

    let (major, minor) = match detect() {
        Some(v) => v,
        None => {
            // No probe resolved a version. opencv-rust may fall back to legacy
            // pre-generated bindings (no `cv::objdetect` ArUco), so guessing
            // modern here can produce a confusing unresolved-import error.
            println!(
                "cargo:warning=shalgalt-cv: could not detect the OpenCV version \
                 (no pkg-config, OPENCV_VERSION, or OPENCV_INCLUDE_PATHS). Assuming \
                 modern (4.7+) cv::objdetect::ArucoDetector. If the build fails with \
                 unresolved `opencv::objdetect` imports, your OpenCV is 4.6 — set \
                 OPENCV_FORCE_ARUCO=legacy (or =modern to silence this)."
            );
            (4, 8)
        }
    };

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
