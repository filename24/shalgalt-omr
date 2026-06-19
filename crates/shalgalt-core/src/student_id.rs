//! Decode the student identifier from a parsed sheet's bubble readings.
//!
//! The Mongolian standard card carries the student code ("Шифр") as a stack of
//! `BubbleKind::StudentId` rows, each a 0–9 digit column. The CV pipeline reads
//! every bubble's fill ratio but stops at "parsed sheet"; turning those readings
//! into a positional digit string is pure logic, so it lives here in the core
//! crate where it is testable without OpenCV.
//!
//! Decoding contract:
//! - Each `BubbleKind::StudentId` group contributes exactly one digit. Because a
//!   cipher row is a "select exactly one of 0–9" structure, the digit is the
//!   **index of the unambiguously darkest bubble** in the row — selected
//!   *relative* to its row-mates, not against the absolute `is_filled` (`> 0.65`)
//!   band used for grading. Real scans of photocopied sheets routinely land a
//!   genuinely-marked cipher bubble right at ~0.65; demanding `> 0.65` made the
//!   whole code unreadable whenever one digit grazed the boundary. Templates must
//!   order the bubbles `0, 1, 2, …` so the index equals the printed digit value.
//! - A row is accepted only when its darkest bubble (a) clears the
//!   confidently-unfilled band (`> 0.35`, so a blank row is rejected) and (b)
//!   leads the runner-up by at least [`MIN_SEPARATION`] (so an erasure or a
//!   genuine double-mark is rejected as ambiguous rather than guessed).
//! - The decode is **all-or-nothing**: if any StudentId row is blank or
//!   ambiguous, the positional code cannot be trusted, so the whole read returns
//!   `None` and the caller falls back to a generated label.
//! - A template with no StudentId group returns `None`.

use crate::domain::{BubbleKind, BubbleReading, OmrTemplate};

/// Minimum fill gap between a cipher row's darkest bubble and its runner-up for
/// the darkest to count as the selected digit. Below this the row reads as an
/// erasure / double-mark and the whole code is rejected as untrustworthy.
const MIN_SEPARATION: f32 = 0.15;

/// Decode the student id text from `readings` against `template`.
///
/// Returns `Some(code)` only when every `StudentId` group yields an unambiguous
/// digit; otherwise `None`. See the module docs for the full contract.
pub fn decode_student_id(template: &OmrTemplate, readings: &[BubbleReading]) -> Option<String> {
    let mut code = String::new();
    let mut saw_student_id_group = false;

    for group in &template.groups {
        if group.kind != BubbleKind::StudentId {
            continue;
        }
        saw_student_id_group = true;

        let digit = best_marked_index(&group.id, readings)?;
        code.push_str(&digit.to_string());
    }

    if !saw_student_id_group {
        return None;
    }
    Some(code)
}

/// The index of the unambiguously darkest bubble for `group_id`, or `None` when
/// the row is blank (darkest still reads as empty paper) or ambiguous (the two
/// darkest bubbles are within [`MIN_SEPARATION`]).
fn best_marked_index(group_id: &str, readings: &[BubbleReading]) -> Option<u32> {
    let mut row: Vec<&BubbleReading> = readings.iter().filter(|r| r.group_id == group_id).collect();
    row.sort_by(|a, b| b.fill.total_cmp(&a.fill));

    let darkest = row.first()?;
    // Blank row: even its darkest bubble sits in the confidently-unfilled band, so
    // there is no mark to read.
    if darkest.fill <= BubbleReading::FILL_UNFILLED_MAX {
        return None;
    }
    // Ambiguous row: a second bubble is nearly as dark (erasure / double-mark).
    let runner_up = row.get(1).map_or(0.0, |r| r.fill);
    if darkest.fill - runner_up < MIN_SEPARATION {
        return None;
    }
    Some(darkest.bubble_index)
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
    fn borderline_filled_marks_decode() {
        // Regression for the reported bug: a real scan landed every cipher mark at
        // ~0.65 — right on the grading "filled" boundary, with one digit dipping
        // just under it. The old `> 0.65` rule rejected the whole code. A
        // select-one row reads the unambiguous darkest mark regardless of the
        // absolute fill level, so this must decode.
        let tpl = template_with(vec![
            group("shifr-0", BubbleKind::StudentId, 10),
            group("shifr-1", BubbleKind::StudentId, 10),
            group("shifr-2", BubbleKind::StudentId, 10),
            group("shifr-3", BubbleKind::StudentId, 10),
        ]);
        let mut readings = Vec::new();
        // Marked bubble at ~0.65 (incl. just below the filled band), rest blank.
        readings.extend((0..10).map(|i| reading("shifr-0", i, if i == 1 { 0.652 } else { 0.05 })));
        readings.extend((0..10).map(|i| reading("shifr-1", i, if i == 0 { 0.68 } else { 0.05 })));
        readings.extend((0..10).map(|i| reading("shifr-2", i, if i == 0 { 0.649 } else { 0.05 })));
        readings.extend((0..10).map(|i| reading("shifr-3", i, if i == 0 { 0.68 } else { 0.05 })));

        assert_eq!(decode_student_id(&tpl, &readings).as_deref(), Some("1000"));
    }

    #[test]
    fn lone_light_mark_is_decoded() {
        // A single bubble marked in the uncertain band with the rest of the row
        // clearly blank is an unambiguous selection — read it rather than falling
        // back. (Grading still flags such a bubble for review; student-id decode is
        // a separate, relative decision.)
        let tpl = template_with(vec![group("shifr-0", BubbleKind::StudentId, 10)]);
        let readings: Vec<_> = (0..10)
            .map(|i| reading("shifr-0", i, if i == 4 { 0.50 } else { 0.05 }))
            .collect();

        assert_eq!(decode_student_id(&tpl, &readings).as_deref(), Some("4"));
    }

    #[test]
    fn ambiguous_two_close_marks_yield_none() {
        // Two bubbles within `MIN_SEPARATION` of each other — an erasure or a
        // double-mark. The row is untrustworthy, so the whole code is rejected.
        let tpl = template_with(vec![group("shifr-0", BubbleKind::StudentId, 10)]);
        let readings: Vec<_> = (0..10)
            .map(|i| {
                reading(
                    "shifr-0",
                    i,
                    if i == 3 {
                        0.62
                    } else if i == 7 {
                        0.55
                    } else {
                        0.05
                    },
                )
            })
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
