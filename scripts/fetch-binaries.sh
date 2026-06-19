#!/usr/bin/env bash
#
# fetch-binaries.sh — download per-OS runtime native libraries into
# apps/desktop/resources/ so the Tauri bundle can ship them.
#
# Currently fetches:
#   - pdfium (pdfium-render's runtime dynamic library) from bblanchon/pdfium-binaries
#
# OpenCV on Windows (opencv_world<NNN>.dll) is handled by fetch-binaries.ps1, since the
# upstream OpenCV Windows artifact is a Windows-only self-extracting .exe. On Linux/macOS
# OpenCV is a build-time system dependency (apt libopencv-dev / brew opencv), so there is
# nothing to copy into resources here.
#
# This script is idempotent: it skips downloads when the target file already exists.
# Pass --force to re-download regardless.
#
# Usage:
#   scripts/fetch-binaries.sh [--force]
#
set -euo pipefail

# --- Configuration ----------------------------------------------------------

# Resolve the repository root from this script's own location so the script works
# regardless of the caller's current working directory.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RESOURCES_DIR="${REPO_ROOT}/apps/desktop/resources"

PDFIUM_BASE_URL="https://github.com/bblanchon/pdfium-binaries/releases/latest/download"

FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
fi

# --- Helpers ----------------------------------------------------------------

log() {
  echo "[fetch-binaries] $*"
}

# Detect the host OS/arch and return the matching pdfium-binaries archive name.
pdfium_archive_for_host() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}" in
    Linux)
      case "${arch}" in
        x86_64 | amd64) echo "pdfium-linux-x64.tgz" ;;
        aarch64 | arm64) echo "pdfium-linux-arm64.tgz" ;;
        *) return 1 ;;
      esac
      ;;
    Darwin)
      case "${arch}" in
        arm64) echo "pdfium-mac-arm64.tgz" ;;
        x86_64) echo "pdfium-mac-x64.tgz" ;;
        *) return 1 ;;
      esac
      ;;
    *)
      return 1
      ;;
  esac
}

# The dynamic library filename produced for the host OS inside the pdfium archive.
pdfium_lib_name_for_host() {
  case "$(uname -s)" in
    Linux) echo "libpdfium.so" ;;
    Darwin) echo "libpdfium.dylib" ;;
    *) return 1 ;;
  esac
}

# --- pdfium ------------------------------------------------------------------

fetch_pdfium() {
  local archive lib_name dest
  if ! archive="$(pdfium_archive_for_host)"; then
    log "ERROR: unsupported OS/arch for pdfium: $(uname -s)/$(uname -m)"
    log "       On Windows, run scripts/fetch-binaries.ps1 instead."
    exit 1
  fi
  lib_name="$(pdfium_lib_name_for_host)"
  dest="${RESOURCES_DIR}/${lib_name}"

  if [[ -f "${dest}" && "${FORCE}" -eq 0 ]]; then
    log "pdfium already present: ${dest} (use --force to re-download)"
    return 0
  fi

  local url tmp_dir
  url="${PDFIUM_BASE_URL}/${archive}"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "${tmp_dir}"' RETURN

  log "Downloading pdfium: ${url}"
  curl -fsSL "${url}" -o "${tmp_dir}/pdfium.tgz"

  log "Extracting ${archive}"
  tar -xzf "${tmp_dir}/pdfium.tgz" -C "${tmp_dir}"

  # The archive layout is bin/<lib> on Linux/macOS. Resolve defensively in case
  # upstream moves it, by searching for the expected library name.
  local extracted
  extracted="$(find "${tmp_dir}" -name "${lib_name}" -type f | head -n 1)"
  if [[ -z "${extracted}" ]]; then
    log "ERROR: ${lib_name} not found inside ${archive}"
    exit 1
  fi

  cp "${extracted}" "${dest}"
  log "Installed pdfium -> ${dest}"
}

# --- Main --------------------------------------------------------------------

main() {
  log "Repository root: ${REPO_ROOT}"
  mkdir -p "${RESOURCES_DIR}"
  log "Resources directory: ${RESOURCES_DIR}"

  fetch_pdfium

  log "Done."
}

main "$@"
