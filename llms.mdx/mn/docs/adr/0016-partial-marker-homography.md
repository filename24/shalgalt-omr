# ADR 0016 — Partial-marker homography (https://filename24.github.io/shalgalt-omr/mn/docs/adr/0016-partial-marker-homography)



<Callout type="success" title="Accepted · 2026-06-19">
  **Deciders:** filename24 · **Related issues:** follow-up to P3-01 (ArUco markers) / ADR 0009 ·
  **Refines:** ADR 0009 — relaxes the implicit "all four markers required" behaviour of the
  perspective-alignment step while keeping every other ADR 0009 decision intact.
</Callout>

## Context [#context]

ADR 0009 adopted four `DICT_6X6_50` ArUco markers (IDs 0..3, TL/TR/BR/BL) as the
perspective-alignment fiducials. The original `perspective.rs` implementation reduced
each detected marker to its **centre point** (averaging the four corners the detector
returns) and then solved an exact perspective transform with
`getPerspectiveTransform`, which consumes exactly four point correspondences.

That made all four markers mandatory: a planar homography has 8 degrees of freedom and
needs four point pairs, so one marker per correspondence means four markers or nothing.
In practice the single most common reason a hand-photographed or photocopied sheet fails
alignment is **one** corner being lost to glare, shadow, a fold, or cropping — and the
"all four required" rule turned that recoverable situation into a hard failure.

The four-marker requirement was never a mathematical necessity; it was a consequence of
throwing away 12 of the 16 corner points the detector already provides.

## Options considered [#options-considered]

| Option                                                | Min markers to align | Robustness to one lost corner | Notes                                             |
| ----------------------------------------------------- | -------------------- | ----------------------------- | ------------------------------------------------- |
| Keep centres + `getPerspectiveTransform` (status quo) | 4                    | none — hard fail              | 4 correspondences, exact fit                      |
| Centres + `findHomography`                            | 4                    | none (still 1 pt/marker)      | least-squares but still needs 4 markers           |
| **Corners + `findHomography` (RANSAC), ≥3 markers**   | 3                    | recovers one lost marker      | 12–16 correspondences, over-determined            |
| Corners + `findHomography`, ≥2 markers                | 2                    | recovers two                  | two markers on one edge are degenerate / unstable |

## Decision [#decision]

<Callout type="info" title="Decision">
  Use each marker's **four corners** as homography correspondences and fit with
  `calib3d::find_homography` (RANSAC, 3 px reprojection threshold). Align whenever **at least
  3 of the 4** markers are detected.
</Callout>

* Each detected marker contributes 4 correspondences, so three markers give 12 points
  spanning three page corners — well-conditioned for a perspective fit. Four markers give
  16 points; RANSAC additionally discards a single mis-located corner.
* The corner ordering returned by ArUco (clockwise from the marker's own top-left,
  `[TL, TR, BR, BL]`) matches the destination corner order computed from the template, so
  correspondences line up by index. Because every marker is printed upright, this mapping
  holds for all four slots.
* Below three markers the homography is too poorly constrained (two markers on the same
  page edge are near-degenerate), so `< 3` remains a `BadRequest` failure.
* Nothing on the printed sheet changes. The corners already exist in every printed
  marker; existing answer sheets keep working and align more reliably.

Destination corner positions are computed in canonical-pixel space directly from the
template's `Marker { position, size }` — no `PaperSpec` is threaded into the CV crate.
The canonical canvas aspect (`WARP_OUT_W:WARP_OUT_H = 1700:2400 ≈ A4 210:297`) already
matches the print paper (an invariant the bubble sampler relies on), so a single square
half-extent `size * 0.5 * min(WARP_OUT_W, WARP_OUT_H)` reproduces the printed marker
geometry to within a fraction of a pixel.

## Consequences [#consequences]

* `crates/shalgalt-cv/src/perspective.rs`: `DetectedMarker` now carries `corners: [Point2f; 4]` instead of a `centre`. `detect_corner_markers` returns
  `Vec<DetectedMarker>` (the detected subset, sorted by id) and fails only below
  `MIN_MARKERS = 3`. `MarkerLayout` stores per-marker canonical-pixel corner
  destinations. `warp_to_canonical` takes a `&[DetectedMarker]` slice and calls
  `find_homography`.
* `crates/shalgalt-cv/src/pipeline.rs` is unchanged — `&Vec<DetectedMarker>` coerces to
  the new `&[DetectedMarker]` parameter, and both call sites already pass `&markers`.
* The user-facing failure copy is unchanged (still a generic "хуудас танигдсангүй"
  toast); only the developer console hint (`src/lib/grade/diagnostics.ts`) is reworded to
  "fewer than 3 of 4 markers".
* A new integration test (`three_markers_still_align`) drops the TL marker and asserts a
  known filled bubble still reads `> 0.65`, which also locks in the corner-ordering
  correspondence. `deliberately_bad` now blacks out two top markers so the
  "fails gracefully" assertion stays deterministic under the ≥3 rule.

## Follow-up [#follow-up]

* Real-scan validation in P8 hardening may revisit whether a 2-diagonal-marker case is
  worth supporting; it is intentionally excluded here to avoid degenerate fits.
* `find_homography`'s RANSAC threshold (3 px) is calibrated against the 1700×2400
  canonical canvas; P8 may tune it against representative phone/MFP fixtures.

<Cards>
  <Card href="/adr/0009-aruco-markers" title="ADR 0009 — ArUco corner markers">
    The four `DICT_6X6_50` markers (IDs 0–3, TL/TR/BR/BL) this ADR realigns from when one is lost.
  </Card>
  <Card href="/adr/0010-confidence-band" title="ADR 0010 — Fill measurement & confidence">
    The per-bubble fill scoring and `> 0.65` threshold the alignment feeds into.
  </Card>
</Cards>
