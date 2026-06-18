//! Decode the student identifier from a parsed sheet's bubble readings.
//!
//! The Mongolian standard card carries the student code ("Шифр") as a stack of
//! `BubbleKind::StudentId` rows, each a 0–9 digit column. The CV pipeline reads
//! every bubble's fill ratio but stops at "parsed sheet"; turning those readings
//! into a positional digit string is pure logic, so it lives here in the core
//! crate where it is testable without OpenCV.
//!
//! Decoding contract:
//! - Each `BubbleKind::StudentId` group contributes exactly one digit. The digit
//!   is the **index** of the group's confidently-filled bubble
//!   (`BubbleReading::is_filled`, i.e. `fill > 0.65`). Templates must order the
//!   bubbles `0, 1, 2, …` so the index equals the printed digit value.
//! - When a group has more than one filled bubble (an erasure, a double mark),
//!   the darkest one (highest `fill`) wins.
//! - The decode is **all-or-nothing**: if any StudentId group is blank or has no
//!   confidently-filled bubble, the positional code cannot be trusted, so the
//!   whole read returns `None` and the caller falls back to a generated label.
//! - A template with no StudentId group returns `None`.

use crate::domain::{BubbleKind, BubbleReading, OmrTemplate};

/// Decode the student id text from `readings` against `template`.
///
/// Returns `Some(code)` only when every `StudentId` group yields a confident
/// digit; otherwise `None`. See the module docs for the full contract.
pub fn decode_student_id(template: &OmrTemplate, readings: &[BubbleReading]) -> Option<String> {
    let mut code = String::new();
    let mut saw_student_id_group = false;

    for group in &template.groups {
        if group.kind != BubbleKind::StudentId {
            continue;
        }
        saw_student_id_group = true;

        let digit = best_filled_index(&group.id, readings)?;
        code.push_str(&digit.to_string());
    }

    if !saw_student_id_group {
        return None;
    }
    Some(code)
}

/// The index of the confidently-filled bubble with the highest fill for `group_id`,
/// or `None` when no reading for the group clears the filled band.
fn best_filled_index(group_id: &str, readings: &[BubbleReading]) -> Option<u32> {
    readings
        .iter()
        .filter(|r| r.group_id == group_id && r.is_filled())
        .max_by(|a, b| a.fill.total_cmp(&b.fill))
        .map(|r| r.bubble_index)
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
                    y: 0.1,
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

    /// Mark exactly one bubble (`digit`) per 0–9 row, leaving the rest empty.
    fn digit_row(group_id: &str, digit: u32) -> Vec<BubbleReading> {
        (0..10)
            .map(|i| reading(group_id, i, if i == digit { 0.92 } else { 0.05 }))
            .collect()
    }

    #[test]
    fn decodes_full_cipher_in_template_order() {
        let tpl = template_with(vec![
            group("shifr-0", BubbleKind::StudentId, 10),
            group("shifr-1", BubbleKind::StudentId, 10),
            group("shifr-2", BubbleKind::StudentId, 10),
            group("shifr-3", BubbleKind::StudentId, 10),
        ]);
        let mut readings = Vec::new();
        readings.extend(digit_row("shifr-0", 0));
        readings.extend(digit_row("shifr-1", 0));
        readings.extend(digit_row("shifr-2", 1));
        readings.extend(digit_row("shifr-3", 2));

        assert_eq!(decode_student_id(&tpl, &readings).as_deref(), Some("0012"));
    }

    #[test]
    fn ignores_question_groups() {
        // A StudentId row plus a question row; only the StudentId row feeds the code.
        let tpl = template_with(vec![
            group("shifr-0", BubbleKind::StudentId, 10),
            group("q-1", BubbleKind::Question, 5),
        ]);
        let mut readings = digit_row("shifr-0", 7);
        readings.extend(vec![reading("q-1", 2, 0.92)]);

        assert_eq!(decode_student_id(&tpl, &readings).as_deref(), Some("7"));
    }

    #[test]
    fn blank_digit_row_yields_none() {
        let tpl = template_with(vec![
            group("shifr-0", BubbleKind::StudentId, 10),
            group("shifr-1", BubbleKind::StudentId, 10),
        ]);
        let mut readings = digit_row("shifr-0", 3);
        // shifr-1 left entirely blank.
        readings.extend((0..10).map(|i| reading("shifr-1", i, 0.05)));

        assert_eq!(decode_student_id(&tpl, &readings), None);
    }

    #[test]
    fn uncertain_only_row_yields_none() {
        // The darkest bubble sits in the uncertain band (not confidently filled),
        // so the row has no trustworthy digit and the whole read is rejected.
        let tpl = template_with(vec![group("shifr-0", BubbleKind::StudentId, 10)]);
        let readings: Vec<_> = (0..10)
            .map(|i| reading("shifr-0", i, if i == 4 { 0.50 } else { 0.05 }))
            .collect();

        assert_eq!(decode_student_id(&tpl, &readings), None);
    }

    #[test]
    fn double_marked_row_takes_the_darkest() {
        // Two bubbles filled in one row (erase + re-mark); the darker one wins.
        let tpl = template_with(vec![group("shifr-0", BubbleKind::StudentId, 10)]);
        let readings = vec![reading("shifr-0", 2, 0.70), reading("shifr-0", 8, 0.95)];

        assert_eq!(decode_student_id(&tpl, &readings).as_deref(), Some("8"));
    }

    #[test]
    fn no_student_id_group_yields_none() {
        let tpl = template_with(vec![group("q-1", BubbleKind::Question, 5)]);
        let readings = vec![reading("q-1", 0, 0.92)];

        assert_eq!(decode_student_id(&tpl, &readings), None);
    }
}
