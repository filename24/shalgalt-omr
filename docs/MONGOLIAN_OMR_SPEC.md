# Mongolian-Standard OMR Card Specification

Locked layout specification for the Mongolian general-education-school answer-sheet preset
that ships as `mongolianStandard` in [`src/lib/templates/mongolianStandard.ts`](../src/lib/templates/mongolianStandard.ts)
and is mirrored in [`crates/shalgalt-pdf/tests/common/preset.rs`](../crates/shalgalt-pdf/tests/common/preset.rs).

This document is the **source of truth** for layout decisions. Any change to the preset
must come with an updated section here in the same PR.

---

## 1. Page split — 25% / 75%

A4 portrait. The card is split vertically:

| Zone   | Normalized Y range | Purpose                                              |
|--------|--------------------|------------------------------------------------------|
| top    | `0.00 – 0.27`      | Header + identification block + instructions         |
| body   | `0.27 – 0.95`      | Answer-marking grids (Section 1 + Section 2)         |
| margin | `0.95 – 1.00`      | Bottom safe zone for printer drift                   |

Page margin (all sides): `12 mm`.

The split ratio is roughly **30 / 70 with margins** so the body grid maximises the number
of answer rows the card can carry on a single page.

---

## 2. Top section (Y `0.00 – 0.27`)

### 2.1 Header (title + meta)

Drawn by [`layout::header`](../crates/shalgalt-pdf/src/layout/header.rs) at the very top.
Title + subtitle + school + teacher, left-aligned, 18 pt title, 12 pt sub.

### 2.2 Шифр — student cipher (left)

| Property         | Value          |
|------------------|----------------|
| Position X       | `0.07`         |
| Position Y start | `0.08`         |
| Row spacing      | `0.025`        |
| Rows             | 4              |
| Bubbles per row  | 10 (digits 0–9)|
| Bubble spacing   | `0.032` (≈ 6 mm pitch — 1:1 ratio with the 3 mm bubble) |

Cipher start X aligns with §3.1 Section 1's left column at `0.07`, so the info zone
(top) and the answer zone (bottom) share a single left-edge grid line. The handwriting
underline auto-clamps to ≈ 5 mm at this X (the available run between page margin and
the cipher row label gap). Last cipher row centre at Y `0.155`.

Each row also carries a **handwriting underline** to its left so graders can fall back
to a manual cipher read when OMR detection fails. The handwriting underline sits at the
bubble's lower edge so the digit baseline aligns with the bubble row.

### 2.3 Хувилбар — variant selector

| Property | Value   |
|----------|---------|
| Y        | `0.19`  |
| Start X  | `0.166` (centred horizontally on the cipher block — cipher span 0.07–0.358, midpoint 0.214; variant first bubble sits at midpoint − 1.5 × spacing) |
| Spacing  | `0.032` |
| Bubbles  | 4 (A/B/C/D) |

Variant bottom edge sits at Y `≈ 0.196`. The body section's parent header
(`1-Р ХЭСЭГ` / `2-Р ХЭСЭГ`) renders in the gap Y `0.21 – 0.23` immediately above Q1 —
Variant + parent header keep ≥ 4 mm vertical clearance. Row label is the bare word
**"Хувилбар"**; the bubble letters convey the choices and a longer suffix would push
the label past the page margin at 10 pt.

### 2.4 САНАМЖ — instructions (right)

Right half of the top section. Multi-line **horizontal** text (NOT rotated). The
pre-translated `PdfOptions::instructions` string is split first on `\n` (hard breaks
preserve numbered items) then word-wrapped to the column width.

| Property      | Value                      |
|---------------|----------------------------|
| Anchor X      | `0.55` normalized (≈ 114 mm) — aligns with Section 2's `startX` so the САНАМЖ block and the answer-area numeric blocks share one right-half grid line |
| Anchor Y      | 22 mm from page top        |
| Header text   | `САНАМЖ` (12 pt)           |
| Body text     | 10 pt                      |
| Default body  | 3 numbered items (folding, variant cross-check, ink/pen rules) |

### 2.5 Student name (top centre)

Reserved Y `0.04 – 0.10` between the header lines and the cipher block. A single empty
underline lets the student write their name. The renderer emits this when
[`PdfOptions::header.title`](../crates/shalgalt-pdf/src/style.rs) is non-empty.

---

## 3. Body section (Y `0.27 – 0.95`)

### 3.1 1-Р ХЭСЭГ — multi-choice (left + centre)

| Property          | Value     |
|-------------------|-----------|
| Question count    | **70**    |
| Layout            | 2 columns × 35 rows |
| Choice labels     | `A B C D E` (5 bubbles per row) |
| Row spacing       | `0.020`   |
| Bubble spacing    | `0.032`   |
| Left column X     | `0.07`    |
| Right column X    | `0.30`    |
| Start Y           | `0.27`    |

Row pitch ≈ 5.46 mm, bubble pitch ≈ 5.95 mm — both 1:1 with the 3 mm bubble within
±0.5 mm. Last row of each column ends at Y `≈ 0.95`.

Row label format: `"<question_number>"` ("1", "2", … "70"), drawn right-aligned to the
left of the first bubble via [`layout::labels::draw_row_label`](../crates/shalgalt-pdf/src/layout/labels.rs).

### 3.2 2-Р ХЭСЭГ — numeric blocks (right)

| Property            | Value     |
|---------------------|-----------|
| Sub-blocks          | 4 (`2.1`, `2.2`, `2.3`, `2.4`) |
| Rows per sub-block  | 8 (a–h)   |
| Bubbles per row     | 10 (digits 0–9) |
| Row spacing         | `0.020`   |
| Block spacing       | `0.180`   |
| Bubble spacing      | `0.032`   |
| Start X             | `0.55`    |
| Start Y             | `0.27`    |

Block 2.4's last row ends at Y `≈ 0.95`. Inter-block PDF gap ≈ 10.9 mm — generous
clearance for the 8 pt sub-block labels (`2.1`/`2.2`/`2.3`/`2.4`).

Each sub-block carries a small **`2.1` / `2.2` / `2.3` / `2.4`** sticker centred above
its first row, plus the parent header **`2-Р ХЭСЭГ`** stacked above the sticker on
block 2.1 only. See [`layout::section_headers`](../crates/shalgalt-pdf/src/layout/section_headers.rs).

### 3.3 Section header positioning rules

- Headers are centred horizontally on the **block's bubble-row span**, not on the first
  bubble alone — this prevents the label from bleeding into the bubbles.
- Шифр and Вариант suppress their headers entirely; the layout itself reads as cipher
  and variant on sight.

---

## 4. Bubble visual style

| Property        | Value     |
|-----------------|-----------|
| Diameter        | `3.0 mm` (≈ 40–50 px at 300 DPI scan) |
| Outline         | `0.3 mm` solid black |
| In-circle digit | ~65 % of diameter (~5.5 pt at 3 mm — within the 6–8 pt OMR target) |
| Row label       | `10 pt` Noto Sans, right-aligned |
| Sidebar header  | `12 pt` (`САНАМЖ` strap) |
| Sidebar body    | `10 pt` |
| Section parent  | `12 pt` (`1-Р ХЭСЭГ` / `2-Р ХЭСЭГ`) |
| Section sub     | `8 pt`  (`2.1` / `2.2` / `2.3` / `2.4`) |
| Page header     | `14 pt` title (left), `9 pt` subtitle / school / teacher (right column right-aligned) |

Bubble pitch keeps a 1:1 edge-gap-to-diameter ratio (≈ 6 mm horizontal, ≈ 5.5 mm
vertical). Smaller pitches risk merging adjacent bubbles into a single contour at the
adaptive-threshold pass; wider pitches eat into row count without recognition gain.

---

## 5. Anchors and alignment markers

Four corner squares at `(0.04, 0.04)`, `(0.96, 0.04)`, `(0.96, 0.96)`, `(0.04, 0.96)`.
Each marker is `0.02` of the page's shorter edge (~4.2 mm at A4). ArUco bit-grid
encoding lands in P3-01.

---

## 6. Group inventory

| Section   | Groups | Bubbles | Notes                                          |
|-----------|--------|---------|------------------------------------------------|
| Шифр      | 4      | 40      | StudentId; row label suppressed                |
| Вариант   | 1      | 4       | Question; row label visible                    |
| 1-Р ХЭСЭГ | 70     | 350     | Question; 5 choices per row                    |
| 2-Р ХЭСЭГ | 32     | 320     | Question (numeric); 8 rows × 4 sub-blocks      |
| **Total** | **107**| **714** |                                                |

---

## 7. Change protocol

This spec is locked. To revise:

1. Open an issue describing the layout change and rationale.
2. Update this document.
3. Update both the TS preset and the Rust test fixture in the **same PR**.
4. Regenerate the golden PDF (`cargo test -p shalgalt-pdf --test golden -- --ignored`).
5. Visually verify by running `cargo run -p shalgalt-pdf --example render_standard` and
   inspecting the output PDF.
