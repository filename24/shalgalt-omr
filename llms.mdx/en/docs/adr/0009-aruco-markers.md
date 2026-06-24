# ADR 0009 — ArUco corner markers (https://filename24.github.io/shalgalt-omr/en/docs/adr/0009-aruco-markers)



<Callout type="success" title="Accepted · 2026-05-10">
  **Deciders:** filename24 · **Related:** P3-01 (ArUco markers) · **Supersedes:** the
  implicit "solid corner squares" decision baked into the original P0/P1 template editor.
</Callout>

## Context [#context]

P0/P1 templates carried four solid corner squares as the visual reference points the
CV pipeline relies on for perspective alignment. Solid squares are simple to print and
easy to find with a contour-based detector, but they have two drawbacks:

1. **No identity.** Four indistinguishable squares cannot tell the CV pipeline which
   one is "top-left." If a teacher feeds a sheet upside down or sideways into a
   scanner, the pipeline either guesses the orientation (brittle) or refuses the
   page (annoying for the teacher).
2. **No error detection.** A scribble in a corner of the page can easily be mistaken
   for the marker. We get no help from the marker geometry.

ArUco markers solve both problems at the cost of slightly larger printed glyphs.
Each marker is a self-identifying 2D barcode with a single integer ID and a
fixed bit pattern. The ArUco detector returns a `(corners, id)` pair per match, so
slot resolution is built into detection.

## Options considered [#options-considered]

| Option                       | Identity   | Error detection                                         | Marker size on A4  | Detector availability         |
| ---------------------------- | ---------- | ------------------------------------------------------- | ------------------ | ----------------------------- |
| Solid corner squares (P0/P1) | none       | none                                                    | \~6 × 6 mm         | bespoke contour detector      |
| QR code (full L1)            | high       | high                                                    | \~25 × 25 mm       | OpenCV `QRCodeDetector`       |
| ArUco DICT\_4X4\_50          | per-marker | minimal — only 16-bit data                              | \~6 × 6 mm (small) | OpenCV `aruco::detectMarkers` |
| **ArUco DICT\_6X6\_50**      | per-marker | one-bit Hamming distance ≥ 4 between dictionary entries | \~8 × 8 mm         | OpenCV `aruco::detectMarkers` |
| ArUco DICT\_7X7\_1000        | per-marker | strong                                                  | \~10 × 10 mm       | OpenCV `aruco::detectMarkers` |

## Decision [#decision]

<Callout type="info" title="Decision">
  Adopt **`DICT_6X6_50` with marker IDs `[0, 1, 2, 3]` in TL/TR/BR/BL order.**
</Callout>

Rationale:

* **DICT\_6X6\_50** is a sweet spot. The 6×6 inner data grid plus the mandatory
  1-cell black border gives an 8×8 cell footprint. At 200 DPI on A4 portrait this
  prints at roughly 8 × 8 mm — small enough to fit comfortably inside the 12 mm
  outer margin we already reserve for markers, large enough that a 200 DPI raster
  retains 4-5 pixels per cell which is well within `ArucoDetector`'s tolerance.
* **IDs 0..3** are the smallest non-overlapping identifier set the dictionary
  exposes. Using consecutive IDs makes the slot-to-position mapping trivial:
  `markers[i]` carries ID `i`, and `i ∈ {0, 1, 2, 3}` maps directly to TL/TR/BR/BL.
* **TL/TR/BR/BL** ordering matches the existing `OmrTemplate.markers: [Marker; 4]`
  convention from `crates/shalgalt-core/src/domain/template.rs`. The grading
  engine, the printpdf renderer, and the perspective-warp routine all consume the
  array in this order.

## Consequences [#consequences]

* The Mongolian-standard preset migrates from `MarkerKind::Square` to
  `MarkerKind::Aruco6x6 { ids: [0, 1, 2, 3] }`. The four-element `ids` field is
  carried verbatim per marker — at PDF render time, slot index `i` indexes the
  `ids` array to pick which canonical bit pattern to paint.
* `crates/shalgalt-pdf/src/layout/markers.rs` ships hardcoded canonical
  `DICT_6X6_50` bit patterns for IDs 0..3 only. Other IDs are not currently
  needed; if a future template requires them, the table must be regenerated from
  OpenCV's predefined dictionary using the helper described in the module-level
  comment.
* `crates/shalgalt-cv/src/perspective.rs` carries both ArUco bindings behind
  cfg gates: legacy `cv::aruco::detectMarkers` for OpenCV 4.6 (Ubuntu / Debian
  apt, older Homebrew bottles) and modern `cv::objdetect::ArucoDetector` for
  OpenCV 4.7+ (Windows self-extractor, current Homebrew, vcpkg).
  `crates/shalgalt-cv/build.rs` probes the local OpenCV version and emits one
  of `opencv_aruco_legacy` / `opencv_aruco_modern`. `OPENCV_FORCE_ARUCO=legacy
  |modern` overrides the probe for environments where it cannot read the
  version (e.g. Windows without pkg-config, vcpkg manifest mode). Both APIs
  accept `DICT_6X6_50`; the assemble-corner-set logic is shared.
* The marker side length on the Mongolian-standard preset bumps from `0.02` to
  `0.04` (normalized) so the printed marker has enough cells of margin to remain
  detectable after a copier round-trip.

## Follow-up [#follow-up]

* Real-scan validation lands in P8 hardening once we have representative MFP and
  phone fixtures from a partner school. Until then, the synthetic fixtures in
  `crates/shalgalt-cv/tests/fixtures/` (P3-09) lock the ArUco detection path
  against regressions.

<Cards>
  <Card href="/adr/0010-confidence-band" title="ADR 0010 — Fill measurement & confidence">
    Consumes the markers' warped output: the per-bubble confidence band and the
    `[0.35, 0.65]` "needs review" decision rule.
  </Card>
  <Card href="/adr/0016-partial-marker-homography" title="ADR 0016 — Partial-marker homography">
    Recovering perspective when fewer than four of these markers are detected.
  </Card>
</Cards>
