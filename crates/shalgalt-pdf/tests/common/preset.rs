//! Mongolian-standard OMR preset — test-only fixture builder.
//!
//! The coordinate constants are kept 1:1 in sync with
//! `src/lib/templates/mongolianStandard.ts`. Changes to either side MUST update the
//! other; until P3-09 lands a fixture suite that cross-validates both, this file is the
//! single source of truth on the Rust side.
//!
//! Both `tests/golden.rs` and `tests/preset_smoke.rs` import this module.

#![allow(dead_code)]

use shalgalt_pdf::domain::{
    BubbleGroup, BubbleKind, Marker, MarkerKind, OmrTemplate, TemplatePoint,
};

// Locked layout — see `docs/MONGOLIAN_OMR_SPEC.md`. Touching these constants requires
// updating the spec doc, the TS preset, and regenerating the golden in the same PR.

const SHIFR_START_Y: f32 = 0.08;
const SHIFR_ROW_SPACING: f32 = 0.025;
const SHIFR_ROWS: usize = 4;
const SHIFR_BUBBLE_START_X: f32 = 0.07;
const SHIFR_BUBBLE_SPACING: f32 = 0.032;
const SHIFR_BUBBLE_COUNT: usize = 10;

const VARIANT_Y: f32 = 0.19;
// Centred horizontally on the cipher block — see TS preset for the derivation.
const VARIANT_START_X: f32 = 0.166;
const VARIANT_SPACING: f32 = 0.032;
const VARIANT_COUNT: usize = 4;

const S1_COUNT: usize = 70;
const S1_PER_COLUMN: usize = 35;
const S1_ROW_SPACING: f32 = 0.020;
const S1_START_Y: f32 = 0.27;
const S1_LEFT_X: f32 = 0.07;
const S1_RIGHT_X: f32 = 0.30;
const S1_BUBBLE_SPACING: f32 = 0.032;
const S1_BUBBLE_COUNT: usize = 5;

const S2_ROWS: usize = 8;
const S2_ROW_SPACING: f32 = 0.020;
const S2_BLOCK_SPACING_Y: f32 = 0.180;
const S2_START_Y: f32 = 0.27;
const S2_START_X: f32 = 0.55;
const S2_BUBBLE_COUNT: usize = 10;
const S2_BUBBLE_SPACING: f32 = 0.032;

const S2_BLOCKS: [&str; 4] = ["2.1", "2.2", "2.3", "2.4"];
const ROW_LABELS_2: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

#[allow(clippy::too_many_arguments)]
fn build_row(
    id_prefix: &str,
    label: &str,
    section: &str,
    kind: BubbleKind,
    origin: TemplatePoint,
    count: usize,
    spacing_x: f32,
    answer_index: Option<u32>,
) -> BubbleGroup {
    let bubbles = (0..count)
        .map(|i| TemplatePoint {
            x: origin.x + spacing_x * i as f32,
            y: origin.y,
        })
        .collect();
    BubbleGroup {
        id: id_prefix.to_string(),
        kind,
        label: label.to_string(),
        bubbles,
        answer_index,
        score: 1.0,
        section: Some(section.to_string()),
    }
}

fn shifr_rows() -> Vec<BubbleGroup> {
    (0..SHIFR_ROWS)
        .map(|r| {
            build_row(
                &format!("shifr-{r}"),
                // Cipher rows render with no row label — see TS preset for rationale.
                "",
                "Шифр",
                BubbleKind::StudentId,
                TemplatePoint {
                    x: SHIFR_BUBBLE_START_X,
                    y: SHIFR_START_Y + SHIFR_ROW_SPACING * r as f32,
                },
                SHIFR_BUBBLE_COUNT,
                SHIFR_BUBBLE_SPACING,
                None,
            )
        })
        .collect()
}

fn variant_row() -> BubbleGroup {
    build_row(
        "variant",
        // Row label suppressed — the section header above already prints "Хувилбар".
        "",
        // Section name matches the TS preset (`STANDARD_SECTIONS.variant = "Хувилбар"`)
        // so both renderers emit the same section header text.
        "Хувилбар",
        BubbleKind::Question,
        TemplatePoint {
            x: VARIANT_START_X,
            y: VARIANT_Y,
        },
        VARIANT_COUNT,
        VARIANT_SPACING,
        None,
    )
}

fn section1_questions() -> Vec<BubbleGroup> {
    (0..S1_COUNT)
        .map(|q| {
            let in_left = q < S1_PER_COLUMN;
            let idx_in_col = if in_left { q } else { q - S1_PER_COLUMN };
            let x = if in_left { S1_LEFT_X } else { S1_RIGHT_X };
            let y = S1_START_Y + S1_ROW_SPACING * idx_in_col as f32;
            build_row(
                &format!("q-{}", q + 1),
                &format!("{}", q + 1),
                "1-Р ХЭСЭГ",
                BubbleKind::Question,
                TemplatePoint { x, y },
                S1_BUBBLE_COUNT,
                S1_BUBBLE_SPACING,
                None,
            )
        })
        .collect()
}

fn section2_block(id: &str, block_index: usize, section: &str) -> Vec<BubbleGroup> {
    let block_start_y = S2_START_Y + S2_BLOCK_SPACING_Y * block_index as f32;
    (0..S2_ROWS)
        .map(|r| {
            build_row(
                &format!("{id}-{}", ROW_LABELS_2[r]),
                ROW_LABELS_2[r],
                section,
                BubbleKind::Question,
                TemplatePoint {
                    x: S2_START_X,
                    y: block_start_y + S2_ROW_SPACING * r as f32,
                },
                S2_BUBBLE_COUNT,
                S2_BUBBLE_SPACING,
                None,
            )
        })
        .collect()
}

/// Mongolian-standard preset (67 groups: 4 Шифр + 1 variant + 30 Section-1 questions
/// + 4×8 Section-2 numeric rows).
pub fn mongolian_standard(title: &str) -> OmrTemplate {
    let marker = |id: &str, x: f32, y: f32| Marker {
        id: id.to_string(),
        position: TemplatePoint { x, y },
        size: 0.02,
        kind: MarkerKind::Square,
    };

    let mut groups = Vec::new();
    groups.extend(shifr_rows());
    groups.push(variant_row());
    groups.extend(section1_questions());
    for (idx, id) in S2_BLOCKS.iter().enumerate() {
        groups.extend(section2_block(id, idx, &format!("2-Р ХЭСЭГ ({id})")));
    }

    OmrTemplate {
        version: OmrTemplate::CURRENT_VERSION,
        title: title.to_string(),
        markers: [
            marker("m-tl", 0.04, 0.04),
            marker("m-tr", 0.96, 0.04),
            marker("m-br", 0.96, 0.96),
            marker("m-bl", 0.04, 0.96),
        ],
        groups,
    }
}
