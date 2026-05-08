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

const SHIFR_START_Y: f32 = 0.155;
const SHIFR_ROW_SPACING: f32 = 0.024;
const SHIFR_ROWS: usize = 4;
const SHIFR_BUBBLE_START_X: f32 = 0.075;
const SHIFR_BUBBLE_SPACING: f32 = 0.0255;
const SHIFR_BUBBLE_COUNT: usize = 10;

const VARIANT_Y: f32 = 0.27;
const VARIANT_START_X: f32 = 0.10;
const VARIANT_SPACING: f32 = 0.026;
const VARIANT_COUNT: usize = 4;

const S1_COUNT: usize = 30;
const S1_PER_COLUMN: usize = 15;
const S1_ROW_SPACING: f32 = 0.02;
const S1_START_Y: f32 = 0.36;
const S1_LEFT_X: f32 = 0.36;
const S1_RIGHT_X: f32 = 0.61;
const S1_BUBBLE_SPACING: f32 = 0.03;
const S1_BUBBLE_COUNT: usize = 4;

const S2_ROWS: usize = 8;
const S2_ROW_SPACING: f32 = 0.018;
const S2_START_Y: f32 = 0.74;
const S2_BUBBLE_COUNT: usize = 10;
const S2_BUBBLE_SPACING: f32 = 0.026;
const S2_LEFT_X: f32 = 0.07;
const S2_RIGHT_X: f32 = 0.55;

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
                &format!("Шифр-{}", r + 1),
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
        "Вариант (A/B/C/D)",
        "Вариант",
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
                &format!("Q{}", q + 1),
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

fn section2_block(id: &str, start_x: f32, section: &str) -> Vec<BubbleGroup> {
    (0..S2_ROWS)
        .map(|r| {
            build_row(
                &format!("{id}-{}", ROW_LABELS_2[r]),
                &format!("{id}.{}", ROW_LABELS_2[r]),
                section,
                BubbleKind::Question,
                TemplatePoint {
                    x: start_x,
                    y: S2_START_Y + S2_ROW_SPACING * r as f32,
                },
                S2_BUBBLE_COUNT,
                S2_BUBBLE_SPACING,
                None,
            )
        })
        .collect()
}

/// Mongolian-standard 51-group preset.
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
    groups.extend(section2_block("2.1", S2_LEFT_X, "2-Р ХЭСЭГ (2.1)"));
    groups.extend(section2_block("2.2", S2_RIGHT_X, "2-Р ХЭСЭГ (2.2)"));

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
