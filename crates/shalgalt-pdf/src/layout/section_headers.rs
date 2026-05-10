//! Per-section headers drawn above the first group of each unique section.
//!
//! Sections are pulled verbatim from [`BubbleGroup::section`]. When a section string
//! follows the convention `"Parent (Sub)"` (e.g. `"2-Р ХЭСЭГ (2.1)"`), the parent label
//! is emitted only once per parent and the sub-block label is emitted at every block —
//! so the reader sees one big "2-Р ХЭСЭГ" above the numeric block area and a small
//! "2.1" / "2.2" / "2.3" / "2.4" sticker above each sub-block.
//!
//! Both labels are centred horizontally on the bubble row of the anchor group so they
//! never bleed into the bubbles. They sit ABOVE the first row with enough Y clearance
//! to not collide with the previous block's last row.
//!
//! Шифр and Вариант do not get section headers — the cipher block + variant row are
//! visually obvious from their layout, and the user explicitly asked for them to stay
//! quiet.

use printpdf::FontId;
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{canvas::Canvas, coords, style::BubbleStyle};

/// Font size for the parent section label (e.g. "1-Р ХЭСЭГ").
const PARENT_FONT_SIZE_PT: f64 = 12.0;

/// Font size for the sub-block label (e.g. "2.1"). Smaller than the parent so it fits
/// inside the inter-block gap without colliding with the previous block's last row.
const SUB_FONT_SIZE_PT: f64 = 8.0;

/// Vertical offset (mm) from the bubble top to the sub-block label baseline. Tuned so
/// the visible glyph top stays clear of the previous block's last row when
/// `blockSpacingY = 0.18` and `rowSpacing = 0.020`.
const SUB_OFFSET_MM: f64 = 2.5;

/// Vertical offset (mm) from the bubble top to the parent label baseline. Sits ABOVE
/// any sub-block label so the two are stacked when both are emitted at the same anchor.
const PARENT_OFFSET_MM: f64 = 9.0;

/// Sections that suppress the header entirely. Шифр stays quiet because the cipher
/// block (4 rows of 0–9 bubbles + handwriting underline) reads as "this is the cipher"
/// at a glance. Хувилбар is **not** suppressed — the section header prints once above
/// the variant row instead of being duplicated as a row label (see TS / Rust preset).
const SUPPRESSED: &[&str] = &["Шифр"];

/// Walk `groups` in order and emit one header (or stacked parent + sub) per unique
/// section. The label is centred horizontally on the anchor group's bubble row so it
/// never spills into the bubbles.
pub fn draw(
    canvas: &mut Canvas,
    groups: &[BubbleGroup],
    paper: &PaperSpec,
    style: &BubbleStyle,
    font: &FontId,
) {
    let r_mm = style.diameter_mm * 0.5;
    let mut last_section: Option<String> = None;
    let mut last_parent: Option<String> = None;

    for group in groups {
        let Some(section) = group.section.as_deref() else {
            continue;
        };
        if last_section.as_deref() == Some(section) {
            continue;
        }
        last_section = Some(section.to_string());

        if SUPPRESSED.contains(&section) {
            continue;
        }

        let Some(first) = group.bubbles.first() else {
            continue;
        };
        let last = group.bubbles.last().unwrap_or(first);
        let (cx_first, cy) = coords::project(first.x as f64, first.y as f64, paper);
        let (cx_last, _) = coords::project(last.x as f64, last.y as f64, paper);
        let row_center_x = (cx_first.0 as f64 + cx_last.0 as f64) / 2.0;
        let bubble_top_y = cy.0 as f64 + r_mm;

        let (parent, sub) = parse_section(section);

        // Sub label: small, centred above the block's first row.
        if let Some(s) = sub {
            canvas.text_centered_x(
                row_center_x,
                bubble_top_y + SUB_OFFSET_MM,
                s,
                font,
                SUB_FONT_SIZE_PT,
            );
        }

        // Parent label: emitted once per parent string. Stacks above the sub label when
        // both anchor at the same group.
        let parent_changed = last_parent.as_deref() != Some(parent);
        if parent_changed {
            let parent_y = bubble_top_y
                + if sub.is_some() {
                    PARENT_OFFSET_MM
                } else {
                    SUB_OFFSET_MM
                };
            canvas.text_centered_x(row_center_x, parent_y, parent, font, PARENT_FONT_SIZE_PT);
            last_parent = Some(parent.to_string());
        }
    }
}

/// Split `"Parent (Sub)"` into `("Parent", Some("Sub"))`. Returns `(s, None)` for any
/// other shape so plain section labels like `"1-Р ХЭСЭГ"` pass through unchanged.
fn parse_section(s: &str) -> (&str, Option<&str>) {
    if let Some(open) = s.find(" (") {
        if let Some(close) = s.rfind(')') {
            if close > open + 2 {
                return (&s[..open], Some(&s[open + 2..close]));
            }
        }
    }
    (s, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_section_extracts_parens() {
        assert_eq!(parse_section("2-Р ХЭСЭГ (2.1)"), ("2-Р ХЭСЭГ", Some("2.1")));
        assert_eq!(parse_section("Parent (Sub)"), ("Parent", Some("Sub")));
    }

    #[test]
    fn parse_section_passes_through_plain_labels() {
        assert_eq!(parse_section("1-Р ХЭСЭГ"), ("1-Р ХЭСЭГ", None));
        assert_eq!(parse_section("Шифр"), ("Шифр", None));
    }

    #[test]
    fn parse_section_handles_empty_parens() {
        assert_eq!(parse_section("Parent ()"), ("Parent ()", None));
    }
}
