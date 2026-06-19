//! Decode the exam-form variant ("Хувилбар") from a parsed sheet's readings.
//!
//! The Mongolian standard card carries a single `BubbleKind::Variant` row whose
//! options print as the choice letters `A, B, C, …`. The student fills one to
//! declare which form they sat. Mapping that mark back to a variant *name* is
//! pure logic, so — like [`crate::decode_student_id`] — it lives in the core
//! crate where it is testable without OpenCV.
//!
//! Decoding contract:
//! - The decoded value is the **printed letter** of the filled bubble: index 0 →
//!   `"A"`, 1 → `"B"`, …. This matches the renderer's default choice labels and
//!   the convention that answer-key `variant` names are those same letters.
//! - Decoding is **strict**: exactly one confidently-filled bubble
//!   (`BubbleReading::is_filled`) yields the letter. Zero filled (blank), more
//!   than one filled (ambiguous double-mark), or a top bubble stuck in the
//!   uncertain band all return `None`. A wrong variant mis-grades the entire
//!   sheet, so ambiguity must fail loudly rather than guess.
//! - A template with no `BubbleKind::Variant` group returns `None`.

use crate::domain::{BubbleKind, BubbleReading, OmrTemplate};

/// Decode the variant name from `readings` against `template`.
///
/// Returns `Some(letter)` only when a single variant bubble is confidently
/// filled; otherwise `None`. See the module docs for the full contract.
pub fn decode_variant(template: &OmrTemplate, readings: &[BubbleReading]) -> Option<String> {
    let group = template
        .groups
        .iter()
        .find(|g| g.kind == BubbleKind::Variant)?;

    let mut filled = readings
        .iter()
        .filter(|r| r.group_id == group.id && r.is_filled());

    let first = filled.next()?;
    // More than one confident mark is an ambiguous double-mark — refuse to guess.
    if filled.next().is_some() {
        return None;
    }
    index_to_letter(first.bubble_index)
}

/// Map a 0-based bubble index to its printed choice letter (`0 → "A"`). Returns
/// `None` for indices past `Z`, which no real variant row reaches.
fn index_to_letter(index: u32) -> Option<String> {
    if index >= 26 {
        return None;
    }
    let ch = (b'A' + index as u8) as char;
    Some(ch.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BubbleGroup, Marker, MarkerKind, OmrTemplate, TemplatePoint};

    fn marker(id: &str, x: f32, y: f32) -> Marker {
        Marker {
            id: id.to_string(),
            position: TemplatePoint { x, y },
            size: 0.04,
            kind: MarkerKind::Square,
        }
    }

    fn group(id: &str, kind: BubbleKind, options: u32) -> BubbleGroup {
        BubbleGroup {
            id: id.to_string(),
            kind,
            label: id.to_string(),
            bubbles: (0..options)
                .map(|i| TemplatePoint {
                    x: 0.1 + i as f32 * 0.03,
                    y: 0.2,
                })
                .collect(),
            answer_index: None,
            score: 0.0,
            section: None,
        }
    }

    fn template_with(groups: Vec<BubbleGroup>) -> OmrTemplate {
        OmrTemplate {
            version: OmrTemplate::CURRENT_VERSION,
            title: "t".into(),
            markers: [
                marker("tl", 0.05, 0.05),
                marker("tr", 0.95, 0.05),
                marker("br", 0.95, 0.95),
                marker("bl", 0.05, 0.95),
            ],
            groups,
        }
    }

    fn reading(group_id: &str, bubble_index: u32, fill: f32) -> BubbleReading {
        BubbleReading {
            group_id: group_id.to_string(),
            bubble_index,
            fill,
            confidence: 0.99,
        }
    }

    fn variant_template() -> OmrTemplate {
        template_with(vec![
            group("variant", BubbleKind::Variant, 5),
            group("q-1", BubbleKind::Question, 5),
        ])
    }

    #[test]
    fn decodes_filled_bubble_to_its_letter() {
        let tpl = variant_template();
        // Bubble index 1 filled → "B".
        let readings = vec![
            reading("variant", 0, 0.05),
            reading("variant", 1, 0.92),
            reading("variant", 2, 0.05),
        ];
        assert_eq!(decode_variant(&tpl, &readings).as_deref(), Some("B"));
    }

    #[test]
    fn first_bubble_decodes_to_a() {
        let tpl = variant_template();
        let readings = vec![reading("variant", 0, 0.92)];
        assert_eq!(decode_variant(&tpl, &readings).as_deref(), Some("A"));
    }

    #[test]
    fn ignores_question_group_marks() {
        let tpl = variant_template();
        // A filled question bubble must not be mistaken for a variant mark.
        let readings = vec![reading("q-1", 2, 0.92), reading("variant", 3, 0.92)];
        assert_eq!(decode_variant(&tpl, &readings).as_deref(), Some("D"));
    }

    #[test]
    fn blank_variant_row_yields_none() {
        let tpl = variant_template();
        let readings = vec![
            reading("variant", 0, 0.05),
            reading("variant", 1, 0.05),
            reading("variant", 2, 0.05),
        ];
        assert_eq!(decode_variant(&tpl, &readings), None);
    }

    #[test]
    fn double_marked_variant_yields_none() {
        // Two confident marks — which form? Refuse to guess.
        let tpl = variant_template();
        let readings = vec![reading("variant", 0, 0.92), reading("variant", 2, 0.95)];
        assert_eq!(decode_variant(&tpl, &readings), None);
    }

    #[test]
    fn uncertain_variant_mark_yields_none() {
        let tpl = variant_template();
        let readings = vec![reading("variant", 1, 0.50)];
        assert_eq!(decode_variant(&tpl, &readings), None);
    }

    #[test]
    fn template_without_variant_group_yields_none() {
        let tpl = template_with(vec![group("q-1", BubbleKind::Question, 5)]);
        let readings = vec![reading("q-1", 0, 0.92)];
        assert_eq!(decode_variant(&tpl, &readings), None);
    }
}
