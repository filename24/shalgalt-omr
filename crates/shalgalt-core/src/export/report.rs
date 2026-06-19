//! Report model + pure aggregation for the xlsx export (P5-01).
//!
//! The host gathers graded sheets out of SQLite (`jobs.graded_sheets_json`) and
//! builds an [`XlsxReport`] plus a [`ReportLabels`] bundle, then hands both to
//! [`crate::export::xlsx::build_report`]. Splitting the data ([`XlsxReport`]) from
//! the human-language text ([`ReportLabels`]) keeps this crate English-only — every
//! Mongolian string is injected by the frontend string-table, never hard-coded here.
//!
//! The aggregation helpers ([`question_stats`], [`error_rows`]) are pure and fully
//! unit-tested; the workbook writer is a thin presentation layer on top of them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::GradedAnswer;

/// One question column of the exam, in display order. `label` is the
/// human-facing question name (e.g. `"1"`, `"2.1.a"`); `group_id` matches the
/// `BubbleGroup.id` carried by each [`GradedAnswer`].
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct QuestionColumn {
    pub group_id: String,
    pub label: String,
    /// Maximum points for this question (the template `BubbleGroup.score`). The
    /// breakdown sheet needs it to turn a graded outcome into earned points via
    /// [`crate::grading::awarded_points`].
    pub score: f64,
}

/// One graded sheet flattened for export. `label` is the resolved student name
/// (roster match), or the raw bubbled id, or a host-supplied fallback — the
/// frontend decides, this crate only prints it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct StudentRow {
    pub label: String,
    /// Exam-form variant the sheet was graded against (the printed letter, e.g.
    /// `"A"`). `None` for single-variant exams or sheets whose variant could not
    /// be resolved; the writer prints an empty cell.
    #[ts(optional, type = "string")]
    pub variant: Option<String>,
    pub total_score: f64,
    pub needs_review: bool,
    pub answers: Vec<GradedAnswer>,
}

/// A full export payload: the exam title, the ordered question set, and one row
/// per graded sheet.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct XlsxReport {
    pub title: String,
    pub questions: Vec<QuestionColumn>,
    pub rows: Vec<StudentRow>,
}

/// Every user-visible string the workbook needs, supplied by the host so this
/// crate carries no Mongolian text. The frontend builds this from the P1
/// string-table (`mn.results.export.*`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct ReportLabels {
    /// Worksheet (tab) names.
    pub summary_sheet: String,
    pub breakdown_sheet: String,
    pub per_question_sheet: String,
    pub errors_sheet: String,
    /// Summary sheet headers.
    pub col_index: String,
    pub col_student: String,
    pub col_variant: String,
    pub col_score: String,
    pub col_status: String,
    /// Per-question sheet headers.
    pub col_question: String,
    pub col_correct: String,
    pub col_wrong: String,
    pub col_blank: String,
    pub col_partial: String,
    pub col_uncertain: String,
    pub col_accuracy: String,
    /// Errors sheet header (student + question columns reuse the headers above).
    pub col_outcome: String,
    /// Status words for the Summary sheet's status column.
    pub status_ok: String,
    pub status_needs_review: String,
    /// Outcome words for the Errors sheet (mirror the non-correct [`OutcomeKind`]s).
    pub outcome_wrong: String,
    pub outcome_blank: String,
    pub outcome_multiple: String,
    pub outcome_partial: String,
    pub outcome_uncertain: String,
    /// Placeholder shown on the Errors sheet when there are no errors.
    pub empty: String,
}

/// Coarse outcome bucket for one graded question. Mirrors the score-bearing
/// [`GradedAnswer`] variants, collapsed to what the export surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeKind {
    Correct,
    Wrong,
    Blank,
    Multiple,
    Partial,
    Uncertain,
}

impl OutcomeKind {
    /// Classify a graded answer into its export bucket.
    pub fn from_answer(answer: &GradedAnswer) -> Self {
        match answer {
            GradedAnswer::Correct { .. } => OutcomeKind::Correct,
            GradedAnswer::Wrong { .. } => OutcomeKind::Wrong,
            GradedAnswer::Blank { .. } => OutcomeKind::Blank,
            GradedAnswer::Multiple { .. } => OutcomeKind::Multiple,
            GradedAnswer::Partial { .. } => OutcomeKind::Partial,
            GradedAnswer::Uncertain { .. } => OutcomeKind::Uncertain,
        }
    }

    /// `true` for every bucket except [`OutcomeKind::Correct`] — i.e. the rows
    /// that belong on the Errors sheet.
    pub fn is_error(self) -> bool {
        !matches!(self, OutcomeKind::Correct)
    }
}

/// The `group_id` an answer belongs to, regardless of variant.
pub(crate) fn answer_group_id(answer: &GradedAnswer) -> &str {
    match answer {
        GradedAnswer::Correct { group_id, .. }
        | GradedAnswer::Wrong { group_id, .. }
        | GradedAnswer::Blank { group_id }
        | GradedAnswer::Multiple { group_id, .. }
        | GradedAnswer::Partial { group_id, .. }
        | GradedAnswer::Uncertain { group_id, .. } => group_id,
    }
}

/// Per-question tally across every row. One entry per [`QuestionColumn`], in the
/// same order. `accuracy` is `correct / graded` in `[0, 1]`; partials are
/// reported in their own column and do **not** count toward `correct`.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestionStat {
    pub group_id: String,
    pub label: String,
    pub correct: u32,
    pub wrong: u32,
    pub blank: u32,
    pub multiple: u32,
    pub partial: u32,
    pub uncertain: u32,
    /// Rows that actually graded this question (i.e. had a matching answer).
    pub graded: u32,
    pub accuracy: f64,
}

/// Tally each question column across all rows.
pub fn question_stats(report: &XlsxReport) -> Vec<QuestionStat> {
    report
        .questions
        .iter()
        .map(|q| {
            let mut stat = QuestionStat {
                group_id: q.group_id.clone(),
                label: q.label.clone(),
                correct: 0,
                wrong: 0,
                blank: 0,
                multiple: 0,
                partial: 0,
                uncertain: 0,
                graded: 0,
                accuracy: 0.0,
            };
            for row in &report.rows {
                let Some(answer) = row
                    .answers
                    .iter()
                    .find(|a| answer_group_id(a) == q.group_id)
                else {
                    continue;
                };
                stat.graded += 1;
                match OutcomeKind::from_answer(answer) {
                    OutcomeKind::Correct => stat.correct += 1,
                    OutcomeKind::Wrong => stat.wrong += 1,
                    OutcomeKind::Blank => stat.blank += 1,
                    OutcomeKind::Multiple => stat.multiple += 1,
                    OutcomeKind::Partial => stat.partial += 1,
                    OutcomeKind::Uncertain => stat.uncertain += 1,
                }
            }
            stat.accuracy = if stat.graded == 0 {
                0.0
            } else {
                stat.correct as f64 / stat.graded as f64
            };
            stat
        })
        .collect()
}

/// One (student, question) cell that scored less than full marks. Ordered by row
/// then by question column so the Errors sheet reads top-to-bottom, left-to-right.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorRow {
    pub student_label: String,
    pub question_label: String,
    pub kind: OutcomeKind,
}

/// Flatten every non-correct graded question into [`ErrorRow`]s. Questions a row
/// did not grade (no matching answer) are skipped — they are not "wrong", just
/// outside that sheet's question set.
pub fn error_rows(report: &XlsxReport) -> Vec<ErrorRow> {
    let mut out = Vec::new();
    for row in &report.rows {
        for q in &report.questions {
            let Some(answer) = row
                .answers
                .iter()
                .find(|a| answer_group_id(a) == q.group_id)
            else {
                continue;
            };
            let kind = OutcomeKind::from_answer(answer);
            if kind.is_error() {
                out.push(ErrorRow {
                    student_label: row.label.clone(),
                    question_label: q.label.clone(),
                    kind,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(id: &str, label: &str) -> QuestionColumn {
        QuestionColumn {
            group_id: id.into(),
            label: label.into(),
            score: 1.0,
        }
    }

    fn correct(id: &str) -> GradedAnswer {
        GradedAnswer::Correct {
            group_id: id.into(),
            marked_indices: vec![0],
        }
    }

    fn wrong(id: &str) -> GradedAnswer {
        GradedAnswer::Wrong {
            group_id: id.into(),
            marked_indices: vec![1],
            correct_indices: vec![0],
        }
    }

    fn blank(id: &str) -> GradedAnswer {
        GradedAnswer::Blank {
            group_id: id.into(),
        }
    }

    fn partial(id: &str) -> GradedAnswer {
        GradedAnswer::Partial {
            group_id: id.into(),
            marked_indices: vec![0],
            correct_indices: vec![0, 1],
            score_ratio: 0.5,
        }
    }

    fn uncertain(id: &str) -> GradedAnswer {
        GradedAnswer::Uncertain {
            group_id: id.into(),
            uncertain_indices: vec![2],
        }
    }

    fn row(label: &str, answers: Vec<GradedAnswer>) -> StudentRow {
        StudentRow {
            label: label.into(),
            variant: None,
            total_score: 0.0,
            needs_review: false,
            answers,
        }
    }

    fn report(questions: Vec<QuestionColumn>, rows: Vec<StudentRow>) -> XlsxReport {
        XlsxReport {
            title: "Test".into(),
            questions,
            rows,
        }
    }

    #[test]
    fn outcome_kind_maps_each_variant() {
        assert_eq!(
            OutcomeKind::from_answer(&correct("q")),
            OutcomeKind::Correct
        );
        assert_eq!(OutcomeKind::from_answer(&wrong("q")), OutcomeKind::Wrong);
        assert_eq!(OutcomeKind::from_answer(&blank("q")), OutcomeKind::Blank);
        assert_eq!(
            OutcomeKind::from_answer(&partial("q")),
            OutcomeKind::Partial
        );
        assert_eq!(
            OutcomeKind::from_answer(&uncertain("q")),
            OutcomeKind::Uncertain
        );
        assert_eq!(
            OutcomeKind::from_answer(&GradedAnswer::Multiple {
                group_id: "q".into(),
                marked_indices: vec![0, 1],
            }),
            OutcomeKind::Multiple
        );
    }

    #[test]
    fn only_correct_is_not_an_error() {
        assert!(!OutcomeKind::Correct.is_error());
        for kind in [
            OutcomeKind::Wrong,
            OutcomeKind::Blank,
            OutcomeKind::Multiple,
            OutcomeKind::Partial,
            OutcomeKind::Uncertain,
        ] {
            assert!(kind.is_error());
        }
    }

    #[test]
    fn question_stats_tally_each_outcome() {
        let r = report(
            vec![col("q1", "1"), col("q2", "2")],
            vec![
                row("A", vec![correct("q1"), wrong("q2")]),
                row("B", vec![correct("q1"), blank("q2")]),
                row("C", vec![wrong("q1"), correct("q2")]),
            ],
        );
        let stats = question_stats(&r);
        assert_eq!(stats.len(), 2);

        let q1 = &stats[0];
        assert_eq!(q1.group_id, "q1");
        assert_eq!(q1.correct, 2);
        assert_eq!(q1.wrong, 1);
        assert_eq!(q1.graded, 3);
        assert!((q1.accuracy - 2.0 / 3.0).abs() < 1e-9);

        let q2 = &stats[1];
        assert_eq!(q2.correct, 1);
        assert_eq!(q2.wrong, 1);
        assert_eq!(q2.blank, 1);
        assert_eq!(q2.graded, 3);
        assert!((q2.accuracy - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn question_stats_skip_ungraded_rows() {
        // Row B did not grade q2 (subset exam): it must not inflate `graded`.
        let r = report(
            vec![col("q1", "1"), col("q2", "2")],
            vec![
                row("A", vec![correct("q1"), correct("q2")]),
                row("B", vec![correct("q1")]),
            ],
        );
        let stats = question_stats(&r);
        assert_eq!(stats[1].graded, 1, "q2 was graded by one row only");
        assert!((stats[1].accuracy - 1.0).abs() < 1e-9);
    }

    #[test]
    fn question_stats_zero_graded_has_zero_accuracy() {
        let r = report(vec![col("q1", "1")], vec![]);
        let stats = question_stats(&r);
        assert_eq!(stats[0].graded, 0);
        assert_eq!(stats[0].accuracy, 0.0);
    }

    #[test]
    fn question_stats_preserve_column_order() {
        let r = report(
            vec![col("qb", "B"), col("qa", "A")],
            vec![row("X", vec![correct("qa"), correct("qb")])],
        );
        let stats = question_stats(&r);
        assert_eq!(stats[0].group_id, "qb");
        assert_eq!(stats[1].group_id, "qa");
    }

    #[test]
    fn error_rows_collect_non_correct_only() {
        let r = report(
            vec![col("q1", "1"), col("q2", "2"), col("q3", "3")],
            vec![
                row("A", vec![correct("q1"), wrong("q2"), blank("q3")]),
                row("B", vec![correct("q1"), correct("q2"), correct("q3")]),
            ],
        );
        let errors = error_rows(&r);
        assert_eq!(errors.len(), 2, "only A's q2 and q3 are errors");
        assert_eq!(errors[0].student_label, "A");
        assert_eq!(errors[0].question_label, "2");
        assert_eq!(errors[0].kind, OutcomeKind::Wrong);
        assert_eq!(errors[1].question_label, "3");
        assert_eq!(errors[1].kind, OutcomeKind::Blank);
    }

    #[test]
    fn error_rows_never_emit_correct() {
        // Invariant relied on by xlsx::outcome_label: the Errors sheet must never
        // receive a Correct outcome.
        let r = report(
            vec![col("q1", "1"), col("q2", "2")],
            vec![
                row("A", vec![correct("q1"), wrong("q2")]),
                row("B", vec![correct("q1"), correct("q2")]),
            ],
        );
        for err in error_rows(&r) {
            assert_ne!(err.kind, OutcomeKind::Correct);
        }
    }

    #[test]
    fn error_rows_ordered_by_row_then_question() {
        let r = report(
            vec![col("q1", "1"), col("q2", "2")],
            vec![
                row("first", vec![wrong("q1"), wrong("q2")]),
                row("second", vec![wrong("q1"), wrong("q2")]),
            ],
        );
        let errors = error_rows(&r);
        let labels: Vec<_> = errors
            .iter()
            .map(|e| (e.student_label.as_str(), e.question_label.as_str()))
            .collect();
        assert_eq!(
            labels,
            vec![
                ("first", "1"),
                ("first", "2"),
                ("second", "1"),
                ("second", "2")
            ]
        );
    }
}
