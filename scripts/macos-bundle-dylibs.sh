#!/usr/bin/env bash
#
# macos-bundle-dylibs.sh — make a built macOS .app self-contained.
#
# The `opencv` crate links the app against Homebrew's OpenCV dylibs, whose
# install_names point at `/opt/homebrew/...` — paths that don't exist on a teacher's
# Mac (no Homebrew). dylibbundler walks the main executable's transitive dylib
# dependencies, copies each non-system dylib into `Contents/Frameworks/`, and rewrites
# every reference to `@executable_path/../Frameworks/`. The matching rpath is baked in
# by `apps/desktop/build.rs`.
#
# pdfium is NOT handled here: pdfium-render dlopen's it at runtime (it is not a link-time
# NEEDED dependency, so dylibbundler can't see it). It ships as a Tauri resource and is
# located via `shalgalt_cv::set_pdfium_dir` from the desktop bootstrap.
#
# Ad-hoc code signing keeps the (unsigned v0) bundle launchable after modification.
# Real Developer ID signing + notarization is deferred until certs exist (master plan
# §6.8/§6.9), at which point the dylibs must be Developer ID-signed inside-out before the
# app is signed.
#
# Usage:
#   scripts/macos-bundle-dylibs.sh <path-to-.app>
#
set -euo pipefail

APP="${1:?usage: macos-bundle-dylibs.sh <path-to-.app>}"
BIN_DIR="$APP/Contents/MacOS"
FRAMEWORKS="$APP/Contents/Frameworks"

if ! command -v dylibbundler >/dev/null 2>&1; then
  echo "error: dylibbundler not found — run 'brew install dylibbundler'" >&2
  exit 1
fi

# A Tauri app has exactly one Mach-O executable in Contents/MacOS.
BIN="$(find "$BIN_DIR" -maxdepth 1 -type f -perm +111 | head -n 1)"
if [ -z "$BIN" ]; then
  echo "error: no executable found in $BIN_DIR" >&2
  exit 1
fi

mkdir -p "$FRAMEWORKS"

# -b bundle+fix deps · -cd create dest dir · -od overwrite · -x file to fix ·
# -d dest dir · -p inner install path · -s extra search paths for Homebrew
# (otool may report @rpath-relative names dylibbundler must resolve).
# /usr/lib and /System are ignored by dylibbundler by default.
dylibbundler \
  -b -cd -od \
  -x "$BIN" \
  -d "$FRAMEWORKS" \
  -p '@executable_path/../Frameworks/' \
  -s /opt/homebrew/opt/opencv/lib \
  -s /opt/homebrew/lib

echo "Post-bundle dependencies of $(basename "$BIN"):"
otool -L "$BIN" | sed 's/^/  /'

# Ad-hoc sign inside-out: nested dylibs first, then the executable. Modifying a bundle
# invalidates any prior signature, so this runs last.
find "$FRAMEWORKS" -name '*.dylib' -print0 | xargs -0 -I{} codesign --force --sign - "{}"
codesign --force --sign - "$BIN"

echo "macos-bundle-dylibs: done."
