//! Pure scoring engine — `(OmrTemplate, ParsedSheet, AnswerKey) -> GradedSheet`.
//!
//! No CV, no DB, no I/O. The engine classifies each `BubbleKind::Question`
//! group into one of the [`GradedAnswer`] variants, sums the awarded score,
//! and flags the sheet `needs_review` when any reading sits in the uncertain
//! band.
//!
//! Decision tree per group (locked by ADR `0006-confidence-band`):
//!
//! 1. Any reading uncertain (`fill ∈ [0.35, 0.65]`) → [`GradedAnswer::Uncertain`].
//!    Score = 0. Sheet-level `needs_review = true`.
//! 2. No filled bubbles → [`GradedAnswer::Blank`]. Score = 0.
//! 3. Single-correct question (`correct_indices.len() == 1`):
//!    - exactly one filled bubble equal to the correct one → [`GradedAnswer::Correct`].
//!    - exactly one filled bubble, different from the correct one → [`GradedAnswer::Wrong`].
//!    - more than one filled bubble → [`GradedAnswer::Multiple`].
//! 4. Multi-correct question (`correct_indices.len() >= 2`):
//!    - filled set == correct set → [`GradedAnswer::Correct`] (full score).
//!    - filled set is a strict subset of correct, no extras → [`GradedAnswer::Partial`]
//!      with `score_ratio = filled.len() / correct.len()`.
//!    - filled set contains any bubble not in correct → [`GradedAnswer::Wrong`].
//!      No partial credit when the student also marked a wrong answer.

use std::collections::BTreeSet;

use crate::domain::{AnswerKey, BubbleKind, GradedAnswer, GradedSheet, OmrTemplate, ParsedSheet};
use crate::error::{AppError, AppResult};

/// Grade one parsed sheet against the template's `Question` groups using the
/// supplied answer key. Returns an error when the answer key is missing an
/// entry for any `Question` group — that is a configuration bug, not a
/// runtime input error, and the caller should fix the data before retrying.
pub fn grade(
    template: &OmrTemplate,
    parsed: &ParsedSheet,
    answer_key: &AnswerKey,
) -> AppResult<GradedSheet> {
    let mut answers: Vec<GradedAnswer> = Vec::new();
    let mut total_score: f32 = 0.0;
    let mut needs_review = false;

    for group in &template.groups {
        if !matches!(group.kind, BubbleKind::Question) {
            continue;
        }

        let group_readings: Vec<_> = parsed
            .readings
            .iter()
            .filter(|r| r.group_id == group.id)
            .collect();

        let uncertain: Vec<u32> = group_readings
            .iter()
            .filter(|r| r.is_uncertain())
            .map(|r| r.bubble_index)
            .collect();
        if !uncertain.is_empty() {
            needs_review = true;
            answers.push(GradedAnswer::Uncertain {
                group_id: group.id.clone(),
                uncertain_indices: sorted_unique(uncertain),
            });
            continue;
        }

        let filled: Vec<u32> = group_readings
            .iter()
            .filter(|r| r.is_filled())
            .map(|r| r.bubble_index)
            .collect();
        let filled = sorted_unique(filled);

        if filled.is_empty() {
            answers.push(GradedAnswer::Blank {
                group_id: group.id.clone(),
            });
            continue;
        }

        let correct = answer_key.correct_for(&group.id).ok_or_else(|| {
            AppError::BadRequest(format!(
                "answer key has no entry for question group '{}'",
                group.id
            ))
        })?;
        let correct: Vec<u32> = sorted_unique(correct.to_vec());

        let (graded, awarded) = classify_question(&group.id, group.score, &filled, &correct);
        total_score += awarded;
        answers.push(graded);
    }

    Ok(GradedSheet {
        template_id: parsed.template_id,
        student_id: None,
        student_id_text: parsed.student_id_text.clone(),
        total_score,
        answers,
        image_path: None,
        needs_review,
    })
}

fn classify_question(
    group_id: &str,
    group_score: f32,
    filled: &[u32],
    correct: &[u32],
) -> (GradedAnswer, f32) {
    debug_assert!(!filled.is_empty(), "Blank case should be handled by caller");

    let single_correct = correct.len() == 1;

    if single_correct {
        if filled.len() > 1 {
            return (
                GradedAnswer::Multiple {
                    group_id: group_id.to_string(),
                    marked_indices: filled.to_vec(),
                },
                0.0,
            );
        }
        if filled[0] == correct[0] {
            return (
                GradedAnswer::Correct {
                    group_id: group_id.to_string(),
                    marked_indices: filled.to_vec(),
                },
                group_score,
            );
        }
        return (
            GradedAnswer::Wrong {
                group_id: group_id.to_string(),
                marked_indices: filled.to_vec(),
                correct_indices: correct.to_vec(),
            },
            0.0,
        );
    }

    let filled_set: BTreeSet<u32> = filled.iter().copied().collect();
    let correct_set: BTreeSet<u32> = correct.iter().copied().collect();
    let has_wrong_extra = filled_set.difference(&correct_set).next().is_some();

    if has_wrong_extra {
        return (
            GradedAnswer::Wrong {
                group_id: group_id.to_string(),
                marked_indices: filled.to_vec(),
                correct_indices: correct.to_vec(),
            },
            0.0,
        );
    }

    if filled_set == correct_set {
        return (
            GradedAnswer::Correct {
                group_id: group_id.to_string(),
                marked_indices: filled.to_vec(),
            },
            group_score,
        );
    }

    let ratio = filled.len() as f32 / correct.len() as f32;
    (
        GradedAnswer::Partial {
            group_id: group_id.to_string(),
            marked_indices: filled.to_vec(),
            correct_indices: correct.to_vec(),
            score_ratio: ratio,
        },
        group_score * ratio,
    )
}

fn sorted_unique(mut indices: Vec<u32>) -> Vec<u32> {
    indices.sort_unstable();
    indices.dedup();
    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AnswerKey, AnswerKeyEntry, BubbleGroup, BubbleKind, BubbleReading, Marker, MarkerKind,
        OmrTemplate, ParsedSheet, TemplatePoint,
    };

    fn marker(id: &str, x: f32, y: f32) -> Marker {
        Marker {
            id: id.to_string(),
            position: TemplatePoint { x, y },
            size: 0.04,
            kind: MarkerKind::Square,
        }
    }

    fn question(id: &str, options: u32, answer_index: Option<u32>, score: f32) -> BubbleGroup {
        BubbleGroup {
            id: id.to_string(),
            kind: BubbleKind::Question,
            label: id.to_string(),
            bubbles: (0..options)
                .map(|i| TemplatePoint {
                    x: 0.1 + i as f32 * 0.05,
                    y: 0.5,
                })
                .collect(),
            answer_index,
            score,
            section: None,
        }
    }

    fn student_id_group(id: &str, options: u32) -> BubbleGroup {
        BubbleGroup {
            id: id.to_string(),
            kind: BubbleKind::StudentId,
            label: id.to_string(),
            bubbles: (0..options)
                .map(|i| TemplatePoint {
                    x: 0.1,
                    y: 0.05 + i as f32 * 0.05,
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
            title: "test".into(),
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

    fn parsed(readings: Vec<BubbleReading>) -> ParsedSheet {
        ParsedSheet {
            template_id: 42,
            page_index: 0,
            student_id_text: None,
            readings,
        }
    }

    fn key_single(group_id: &str, correct: u32) -> AnswerKeyEntry {
        AnswerKeyEntry {
            group_id: group_id.to_string(),
            correct_indices: vec![correct],
        }
    }

    fn key_multi(group_id: &str, correct: Vec<u32>) -> AnswerKeyEntry {
        AnswerKeyEntry {
            group_id: group_id.to_string(),
            correct_indices: correct,
        }
    }

    fn answer_key(entries: Vec<AnswerKeyEntry>) -> AnswerKey {
        AnswerKey {
            exam_id: 1,
            variant: "A".into(),
            answers: entries,
        }
    }

    // ---- single-correct path -------------------------------------------------

    #[test]
    fn correct_single_choice_awards_full_score() {
        let tpl = template_with(vec![question("q1", 4, Some(2), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.05),
            reading("q1", 1, 0.10),
            reading("q1", 2, 0.92), // filled (correct)
            reading("q1", 3, 0.08),
        ]);
        let key = answer_key(vec![key_single("q1", 2)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 1.0);
        assert!(!result.needs_review);
        assert_eq!(result.answers.len(), 1);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Correct {
                group_id: "q1".into(),
                marked_indices: vec![2],
            }
        );
    }

    #[test]
    fn wrong_single_choice_scores_zero() {
        let tpl = template_with(vec![question("q1", 4, Some(2), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.92), // filled (wrong)
            reading("q1", 1, 0.10),
            reading("q1", 2, 0.10),
            reading("q1", 3, 0.10),
        ]);
        let key = answer_key(vec![key_single("q1", 2)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 0.0);
        assert!(!result.needs_review);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Wrong {
                group_id: "q1".into(),
                marked_indices: vec![0],
                correct_indices: vec![2],
            }
        );
    }

    #[test]
    fn blank_when_no_bubble_filled() {
        let tpl = template_with(vec![question("q1", 4, Some(2), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.05),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![key_single("q1", 2)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 0.0);
        assert!(!result.needs_review);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Blank {
                group_id: "q1".into(),
            }
        );
    }

    #[test]
    fn multiple_marks_for_single_correct_question_scores_zero() {
        let tpl = template_with(vec![question("q1", 4, Some(2), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.92),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.92), // correct, but extra mark also
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![key_single("q1", 2)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 0.0);
        assert!(!result.needs_review);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Multiple {
                group_id: "q1".into(),
                marked_indices: vec![0, 2],
            }
        );
    }

    // ---- multi-correct path --------------------------------------------------

    #[test]
    fn multi_correct_full_match_awards_full_score() {
        let tpl = template_with(vec![question("q1", 4, None, 2.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.05),
            reading("q1", 1, 0.92), // filled
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.92), // filled
        ]);
        let key = answer_key(vec![key_multi("q1", vec![1, 3])]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 2.0);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Correct {
                group_id: "q1".into(),
                marked_indices: vec![1, 3],
            }
        );
    }

    #[test]
    fn multi_correct_partial_subset_awards_proportional_score() {
        let tpl = template_with(vec![question("q1", 4, None, 2.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.05),
            reading("q1", 1, 0.92), // filled (one of two correct)
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![key_multi("q1", vec![1, 3])]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert!((result.total_score - 1.0).abs() < f32::EPSILON);
        match &result.answers[0] {
            GradedAnswer::Partial {
                group_id,
                marked_indices,
                correct_indices,
                score_ratio,
            } => {
                assert_eq!(group_id, "q1");
                assert_eq!(marked_indices, &vec![1]);
                assert_eq!(correct_indices, &vec![1, 3]);
                assert!((score_ratio - 0.5).abs() < f32::EPSILON);
            }
            other => panic!("expected Partial, got {other:?}"),
        }
    }

    #[test]
    fn multi_correct_with_wrong_extra_scores_zero() {
        let tpl = template_with(vec![question("q1", 4, None, 2.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.92), // filled (not in correct set)
            reading("q1", 1, 0.92), // filled (correct)
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.92), // filled (correct)
        ]);
        let key = answer_key(vec![key_multi("q1", vec![1, 3])]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 0.0);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Wrong {
                group_id: "q1".into(),
                marked_indices: vec![0, 1, 3],
                correct_indices: vec![1, 3],
            }
        );
    }

    // ---- uncertain band ------------------------------------------------------

    #[test]
    fn uncertain_bubble_flags_needs_review_and_scores_zero() {
        let tpl = template_with(vec![question("q1", 4, Some(2), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.05),
            reading("q1", 1, 0.50), // uncertain (in [0.35, 0.65])
            reading("q1", 2, 0.92),
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![key_single("q1", 2)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 0.0);
        assert!(result.needs_review);
        assert_eq!(
            result.answers[0],
            GradedAnswer::Uncertain {
                group_id: "q1".into(),
                uncertain_indices: vec![1],
            }
        );
    }

    #[test]
    fn band_boundaries_are_inclusive_uncertain() {
        let tpl = template_with(vec![question("q1", 4, Some(0), 1.0)]);
        let key = answer_key(vec![key_single("q1", 0)]);

        // fill == 0.35 ⇒ not unfilled (< 0.35), not filled (> 0.65) ⇒ uncertain
        let p = parsed(vec![
            reading("q1", 0, 0.35),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let result = grade(&tpl, &p, &key).unwrap();
        assert!(result.needs_review);
        assert!(matches!(result.answers[0], GradedAnswer::Uncertain { .. }));

        // fill == 0.65 ⇒ also uncertain (boundary inclusive)
        let p = parsed(vec![
            reading("q1", 0, 0.65),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let result = grade(&tpl, &p, &key).unwrap();
        assert!(result.needs_review);
        assert!(matches!(result.answers[0], GradedAnswer::Uncertain { .. }));
    }

    // ---- mixed sheets / passthrough -----------------------------------------

    #[test]
    fn student_id_groups_are_skipped_and_text_is_passed_through() {
        let tpl = template_with(vec![
            student_id_group("sid", 10),
            question("q1", 4, Some(0), 1.0),
        ]);
        let p = ParsedSheet {
            template_id: 7,
            page_index: 0,
            student_id_text: Some("00123".into()),
            readings: vec![
                reading("sid", 0, 0.92),
                reading("q1", 0, 0.92),
                reading("q1", 1, 0.05),
                reading("q1", 2, 0.05),
                reading("q1", 3, 0.05),
            ],
        };
        let key = answer_key(vec![key_single("q1", 0)]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.template_id, 7);
        assert_eq!(result.student_id_text.as_deref(), Some("00123"));
        assert!(result.student_id.is_none());
        assert_eq!(result.answers.len(), 1, "student_id group is not graded");
        assert!(matches!(result.answers[0], GradedAnswer::Correct { .. }));
        assert_eq!(result.total_score, 1.0);
    }

    #[test]
    fn mixed_sheet_aggregates_per_group_scores() {
        let tpl = template_with(vec![
            question("q1", 4, Some(0), 1.0),
            question("q2", 4, Some(1), 2.0),
            question("q3", 4, None, 1.0),
        ]);
        let p = parsed(vec![
            // q1 correct
            reading("q1", 0, 0.92),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
            // q2 wrong
            reading("q2", 0, 0.92),
            reading("q2", 1, 0.05),
            reading("q2", 2, 0.05),
            reading("q2", 3, 0.05),
            // q3 blank
            reading("q3", 0, 0.05),
            reading("q3", 1, 0.05),
            reading("q3", 2, 0.05),
            reading("q3", 3, 0.05),
        ]);
        let key = answer_key(vec![
            key_single("q1", 0),
            key_single("q2", 1),
            key_single("q3", 2),
        ]);

        let result = grade(&tpl, &p, &key).unwrap();

        assert_eq!(result.total_score, 1.0);
        assert!(!result.needs_review);
        assert!(matches!(result.answers[0], GradedAnswer::Correct { .. }));
        assert!(matches!(result.answers[1], GradedAnswer::Wrong { .. }));
        assert!(matches!(result.answers[2], GradedAnswer::Blank { .. }));
    }

    #[test]
    fn missing_answer_key_entry_returns_bad_request() {
        let tpl = template_with(vec![question("q1", 4, Some(0), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.92),
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![]);

        let err = grade(&tpl, &p, &key).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
        assert!(err.to_string().contains("q1"));
    }

    // ---- small helpers / hygiene --------------------------------------------

    #[test]
    fn duplicate_filled_indices_are_deduplicated() {
        // Defensive: even if the CV pipeline ever emitted duplicate readings
        // for the same bubble, the engine should not double-count them.
        let tpl = template_with(vec![question("q1", 4, Some(0), 1.0)]);
        let p = parsed(vec![
            reading("q1", 0, 0.92),
            reading("q1", 0, 0.92), // duplicate index
            reading("q1", 1, 0.05),
            reading("q1", 2, 0.05),
            reading("q1", 3, 0.05),
        ]);
        let key = answer_key(vec![key_single("q1", 0)]);

        let result = grade(&tpl, &p, &key).unwrap();
        assert_eq!(
            result.answers[0],
            GradedAnswer::Correct {
                group_id: "q1".into(),
                marked_indices: vec![0],
            }
        );
        assert_eq!(result.total_score, 1.0);
    }

    #[test]
    fn readings_for_other_groups_do_not_leak_into_classification() {
        // Two questions; only readings for q2 are present (q1 has no readings
        // at all). q1 must be Blank, not crash on missing data.
        let tpl = template_with(vec![
            question("q1", 4, Some(0), 1.0),
            question("q2", 4, Some(1), 1.0),
        ]);
        let p = parsed(vec![
            reading("q2", 1, 0.92),
            reading("q2", 0, 0.05),
            reading("q2", 2, 0.05),
            reading("q2", 3, 0.05),
        ]);
        let key = answer_key(vec![key_single("q1", 0), key_single("q2", 1)]);

        let result = grade(&tpl, &p, &key).unwrap();
        assert!(matches!(result.answers[0], GradedAnswer::Blank { .. }));
        assert!(matches!(result.answers[1], GradedAnswer::Correct { .. }));
        assert_eq!(result.total_score, 1.0);
    }

    #[test]
    fn bubble_reading_band_helpers_are_exclusive() {
        let r = reading("q", 0, 0.34);
        assert!(r.is_unfilled());
        assert!(!r.is_uncertain());
        assert!(!r.is_filled());

        let r = reading("q", 0, 0.66);
        assert!(!r.is_unfilled());
        assert!(!r.is_uncertain());
        assert!(r.is_filled());

        let r = reading("q", 0, 0.50);
        assert!(!r.is_unfilled());
        assert!(r.is_uncertain());
        assert!(!r.is_filled());
    }
}
