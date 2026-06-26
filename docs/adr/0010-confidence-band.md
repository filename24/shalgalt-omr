---
title: ADR 0010 — Fill measurement & confidence
description: Per-bubble fill scoring on the warped canvas, the confidence formula, and the locked [0.35, 0.65] needs-review bands.
---

<Callout type="success" title="Accepted · 2026-05-10">
  **Deciders:** filename24 · **Related:** P3-02 (adaptive thresholding), P3-04 (per-bubble
  confidence).
</Callout>

## Context

Once the page is warped to a canonical canvas (ADR 0009), the CV pipeline must turn
each bubble into a single `(fill, confidence)` reading. The grading engine
(`crates/shalgalt-core/src/grading/engine.rs`) already implements the consumer side:
it expects a `BubbleReading { group_id, bubble_index, fill, confidence }` per bubble
and classifies them into the `[<0.35, 0.35..=0.65, >0.65]` bands.

Two engineering questions remain:

1. How do we measure `fill` from the warped image?
2. What does `confidence` mean, and how is it computed?

## Decision — measurement

<Callout type="info" title="Decision">
  Measure `fill` as the mean ink fraction inside a sample disc strictly within the printed
  bubble, taken from the continuous `255 − gray` signal — **not** from an
  adaptive-threshold binary.
</Callout>

For each bubble:

1. Convert the bubble's normalized centre to canvas pixel coordinates against the
   warped canonical image (1700 × 2400 px).
2. Sample a disc of radius `BUBBLE_RADIUS_FRACTION × canvas_width = 0.010 × 1700 ≈
   17 px`. The bubble itself is printed at radius `0.012 × canvas_width ≈ 20 px`,
   so the sample disc is strictly inside the printed circle and never picks up
   the ring stroke.
3. Compute `fill = mean(255 − gray) / 255` inside the sample disc, where `gray` is
   the warped 8U grayscale image. Result is in `[0, 1]`.

We **do not** apply adaptive thresholding before measurement, even though
adaptive thresholding is the right preprocessing for line-art / outline detection
elsewhere in the pipeline. The reason is documented in
`crates/shalgalt-cv/src/threshold.rs`: a 41-pixel-block adaptive Gaussian
threshold tracks the local mean, and a uniformly filled disc raises the local
mean enough that the *centre* of the disc falls below threshold and is classified
as paper. The continuous `255 − gray` signal preserves the disc's mean intensity,
so a fully filled bubble reads ≈ 1.0 instead of ≈ 0.05.

We **do** apply adaptive thresholding for the structural pre-warp pass
(`threshold::binarize`) used by the Hough deskew (ADR 0009 follow-up: `deskew.rs`).
Adaptive thresholding is the right tool when the input is text, a bubble outline,
or any thin line-art structure that an MFP scan can degrade.

## Decision — confidence

`confidence(fill) = clamp(2 × |fill − 0.5|, 0, 1)`.

Boundary readings:

| `fill` | `confidence` | Meaning |
| ------ | ------------ | ------- |
| `0.00` | `1.00` | Solidly empty paper — no doubt. |
| `0.35` | `0.30` | Lower band edge — entering uncertain territory. |
| `0.50` | `0.00` | Half-filled — equiprobable; the manual reviewer must decide. |
| `0.65` | `0.30` | Upper band edge. |
| `1.00` | `1.00` | Solidly filled — no doubt. |

Confidence is **not** consumed by the grading engine today. The engine owns the
`fill ∈ [0.35, 0.65] ⇒ Uncertain ⇒ needs_review` rule and that decision is
deterministic and band-driven, not confidence-driven (see
`crates/shalgalt-core/src/grading/engine.rs` lines 53–66 for the implementation
reference). Confidence exists for the **manual-review queue** (P3-07): when a
sheet has multiple uncertain bubbles, the queue surfaces low-confidence bubbles
first so the reviewer can clear the most ambiguous answers fastest.

## Decision — band thresholds

The `[<0.35, 0.35..=0.65, >0.65]` bands are inherited from
`crates/shalgalt-core/src/domain/parsed.rs`'s
`BubbleReading::FILL_UNFILLED_MAX = 0.35` and `FILL_FILLED_MIN = 0.65` constants.
Both boundaries are inclusive on the *uncertain* side per the existing grading-
engine tests:

```rust
// engine.rs::tests::band_boundaries_are_inclusive_uncertain
//   fill == 0.35 ⇒ Uncertain (not unfilled)
//   fill == 0.65 ⇒ Uncertain (not filled)
```

This ADR locks the bands; reopening requires a new ADR plus a master-plan update.

## Consequences

- `crates/shalgalt-cv/src/bubbles.rs` reads from the bg-subtracted "ink density"
  signal exposed as `threshold::flatten_to_ink`, not from the adaptive-threshold
  binary. The two threshold variants share a single module and the doc string
  on `binarize` calls out the reason.
- `BUBBLE_RADIUS_FRACTION = 0.010` is calibrated against the Mongolian-standard
  preset's printed bubble radius (`0.012` normalized). Templates with
  significantly different bubble dimensions need a per-template override or a
  template-aware sampling radius (out of scope for P3; revisit in P8 hardening).
- The synthetic fixtures in `crates/shalgalt-cv/tests/fixtures/` (P3-09) drive
  `fill > 0.65` for filled bubbles and `fill < 0.35` for empty bubbles in the
  clean-MFP path, validating the band boundaries end-to-end.

## Follow-up

- Real-scan calibration in P8 hardening will tune `BUBBLE_RADIUS_FRACTION` and
  may revisit the band thresholds against an empirical distribution of fill
  ratios from real student answers.
- Per-bubble confidence is consumed by the manual-review queue (P3-07). The
  ordering rule (`asc_by_confidence`) lands with that queue's UI.

<Cards>
  <Card href="/adr/0009-aruco-markers" title="ADR 0009 — ArUco corner markers">
    The marker-driven perspective warp that produces the canonical canvas this
    measurement samples.
  </Card>
  <Card href="/adr/0008-bubble-label-position" title="ADR 0008 — Bubble label position">
    Why the in-circle printed label stays well below the 0.35 fill threshold.
  </Card>
</Cards>
