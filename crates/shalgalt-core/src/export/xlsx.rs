//! Multi-worksheet Excel output via `rust_xlsxwriter` (P5-01).
//!
//! [`build_report`] turns an [`XlsxReport`] + [`ReportLabels`] into an in-memory
//! `.xlsx` buffer with three sheets:
//!
//! 1. **Summary** — one row per graded sheet: index, student, total score, review
//!    status. A conditional format paints zero scores red.
//! 2. **Per-question** — one row per question: correct / wrong / blank / partial /
//!    uncertain counts and an accuracy percentage. Zero-correct questions are
//!    painted red.
//! 3. **Errors** — one row per (student, question) that scored below full marks,
//!    with the outcome word.
//!
//! The writer holds no human-language text: every label comes from
//! [`ReportLabels`], so the workbook ships Mongolian headers while this crate
//! stays English-only. The buffer is returned to the host, which writes it to the
//! user-chosen path (Rule 1 — no bytes cross the IPC boundary; the file is written
//! Rust-side).

use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatCellRule, Format, FormatAlign, FormatBorder,
    Workbook, XlsxError,
};

use crate::error::{AppError, AppResult};

use super::report::{error_rows, question_stats, OutcomeKind, ReportLabels, XlsxReport};

fn to_app(err: XlsxError) -> AppError {
    AppError::Internal(anyhow::anyhow!("xlsx export failed: {err}"))
}

/// Render the report to an in-memory `.xlsx` buffer.
pub fn build_report(report: &XlsxReport, labels: &ReportLabels) -> AppResult<Vec<u8>> {
    let mut workbook = Workbook::new();

    let header = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xE8E8E8));
    let title = Format::new().set_bold().set_font_size(14);
    let zero_red = Format::new()
        .set_font_color(Color::RGB(0x9C0006))
        .set_background_color(Color::RGB(0xFFC7CE));
    let percent = Format::new().set_num_format("0.0%");
    let muted = Format::new()
        .set_italic()
        .set_font_color(Color::RGB(0x808080));

    write_summary(&mut workbook, report, labels, &title, &header, &zero_red).map_err(to_app)?;
    write_per_question(&mut workbook, report, labels, &header, &percent, &zero_red)
        .map_err(to_app)?;
    write_errors(&mut workbook, report, labels, &header, &muted).map_err(to_app)?;

    workbook.save_to_buffer().map_err(to_app)
}

fn write_summary(
    workbook: &mut Workbook,
    report: &XlsxReport,
    labels: &ReportLabels,
    title: &Format,
    header: &Format,
    zero_red: &Format,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name(&labels.summary_sheet)?;
    sheet.set_column_width(0, 6)?;
    sheet.set_column_width(1, 28)?;
    sheet.set_column_width(2, 12)?;
    sheet.set_column_width(3, 18)?;

    sheet.write_string_with_format(0, 0, &report.title, title)?;

    const HEADER_ROW: u32 = 1;
    sheet.write_string_with_format(HEADER_ROW, 0, &labels.col_index, header)?;
    sheet.write_string_with_format(HEADER_ROW, 1, &labels.col_student, header)?;
    sheet.write_string_with_format(HEADER_ROW, 2, &labels.col_score, header)?;
    sheet.write_string_with_format(HEADER_ROW, 3, &labels.col_status, header)?;

    let first_data = HEADER_ROW + 1;
    for (i, row) in report.rows.iter().enumerate() {
        let r = first_data + i as u32;
        sheet.write_number(r, 0, (i + 1) as f64)?;
        sheet.write_string(r, 1, &row.label)?;
        sheet.write_number(r, 2, row.total_score)?;
        let status = if row.needs_review {
            &labels.status_needs_review
        } else {
            &labels.status_ok
        };
        sheet.write_string(r, 3, status)?;
    }

    // Conditional format: zero scores stand out (master plan P5-01 "red on zeros").
    if !report.rows.is_empty() {
        let last = first_data + report.rows.len() as u32 - 1;
        let rule = ConditionalFormatCell::new()
            .set_rule(ConditionalFormatCellRule::EqualTo(0.0))
            .set_format(zero_red);
        sheet.add_conditional_format(first_data, 2, last, 2, &rule)?;
    }

    Ok(())
}

fn write_per_question(
    workbook: &mut Workbook,
    report: &XlsxReport,
    labels: &ReportLabels,
    header: &Format,
    percent: &Format,
    zero_red: &Format,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name(&labels.per_question_sheet)?;

    let headers = [
        &labels.col_question,
        &labels.col_correct,
        &labels.col_wrong,
        &labels.col_blank,
        &labels.col_partial,
        &labels.col_uncertain,
        &labels.col_accuracy,
    ];
    // First column holds the question label (wider); the rest are narrow counts.
    sheet.set_column_width(0, 16)?;
    for col in 1..headers.len() as u16 {
        sheet.set_column_width(col, 12)?;
    }
    for (col, text) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, text.as_str(), header)?;
    }

    let stats = question_stats(report);
    for (i, stat) in stats.iter().enumerate() {
        let r = 1 + i as u32;
        sheet.write_string(r, 0, &stat.label)?;
        sheet.write_number(r, 1, stat.correct as f64)?;
        sheet.write_number(r, 2, stat.wrong as f64)?;
        sheet.write_number(r, 3, stat.blank as f64)?;
        sheet.write_number(r, 4, stat.partial as f64)?;
        sheet.write_number(r, 5, stat.uncertain as f64)?;
        sheet.write_number_with_format(r, 6, stat.accuracy, percent)?;
    }

    // A question nobody answered correctly is worth flagging. Data rows occupy
    // 1..=stats.len() (header is row 0), so the last data row index == len.
    if !stats.is_empty() {
        let last = stats.len() as u32;
        let rule = ConditionalFormatCell::new()
            .set_rule(ConditionalFormatCellRule::EqualTo(0.0))
            .set_format(zero_red);
        sheet.add_conditional_format(1, 1, last, 1, &rule)?;
    }

    Ok(())
}

fn write_errors(
    workbook: &mut Workbook,
    report: &XlsxReport,
    labels: &ReportLabels,
    header: &Format,
    muted: &Format,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name(&labels.errors_sheet)?;
    sheet.set_column_width(0, 28)?;
    sheet.set_column_width(1, 16)?;
    sheet.set_column_width(2, 16)?;

    sheet.write_string_with_format(0, 0, &labels.col_student, header)?;
    sheet.write_string_with_format(0, 1, &labels.col_question, header)?;
    sheet.write_string_with_format(0, 2, &labels.col_outcome, header)?;

    let errors = error_rows(report);
    if errors.is_empty() {
        sheet.write_string_with_format(1, 0, &labels.empty, muted)?;
        return Ok(());
    }

    for (i, err) in errors.iter().enumerate() {
        let r = 1 + i as u32;
        sheet.write_string(r, 0, &err.student_label)?;
        sheet.write_string(r, 1, &err.question_label)?;
        sheet.write_string(r, 2, outcome_label(err.kind, labels))?;
    }

    Ok(())
}

/// Map an outcome bucket to its host-supplied word. [`OutcomeKind::Correct`]
/// never reaches the Errors sheet (`error_rows` filters it via `is_error()`); the
/// `debug_assert` catches a regression there, and in release it falls back to the
/// wrong-answer label rather than panicking on user data.
fn outcome_label(kind: OutcomeKind, labels: &ReportLabels) -> &str {
    debug_assert_ne!(
        kind,
        OutcomeKind::Correct,
        "Correct answers must not reach the Errors sheet"
    );
    match kind {
        OutcomeKind::Wrong | OutcomeKind::Correct => &labels.outcome_wrong,
        OutcomeKind::Blank => &labels.outcome_blank,
        OutcomeKind::Multiple => &labels.outcome_multiple,
        OutcomeKind::Partial => &labels.outcome_partial,
        OutcomeKind::Uncertain => &labels.outcome_uncertain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::GradedAnswer;
    use crate::export::report::{QuestionColumn, StudentRow};

    // ASCII fixtures only — `shalgalt-core` is English-only, tests included. The
    // real Mongolian labels are injected by the frontend at runtime; here we only
    // verify the writer wires whatever strings it is handed into the workbook.
    fn labels() -> ReportLabels {
        ReportLabels {
            summary_sheet: "summary".into(),
            per_question_sheet: "per_question".into(),
            errors_sheet: "errors".into(),
            col_index: "no".into(),
            col_student: "student".into(),
            col_score: "score".into(),
            col_status: "status".into(),
            col_question: "question".into(),
            col_correct: "correct".into(),
            col_wrong: "wrong".into(),
            col_blank: "blank".into(),
            col_partial: "partial".into(),
            col_uncertain: "uncertain".into(),
            col_accuracy: "accuracy".into(),
            col_outcome: "outcome".into(),
            status_ok: "ok".into(),
            status_needs_review: "review".into(),
            outcome_wrong: "wrong".into(),
            outcome_blank: "blank".into(),
            outcome_multiple: "multiple".into(),
            outcome_partial: "partial".into(),
            outcome_uncertain: "uncertain".into(),
            empty: "no_errors".into(),
        }
    }

    fn sample() -> XlsxReport {
        XlsxReport {
            title: "sample exam".into(),
            questions: vec![
                QuestionColumn {
                    group_id: "q1".into(),
                    label: "1".into(),
                },
                QuestionColumn {
                    group_id: "q2".into(),
                    label: "2".into(),
                },
            ],
            rows: vec![
                StudentRow {
                    label: "student-a".into(),
                    total_score: 2.0,
                    needs_review: false,
                    answers: vec![
                        GradedAnswer::Correct {
                            group_id: "q1".into(),
                            marked_indices: vec![0],
                        },
                        GradedAnswer::Correct {
                            group_id: "q2".into(),
                            marked_indices: vec![1],
                        },
                    ],
                },
                StudentRow {
                    label: "student-b".into(),
                    total_score: 0.0,
                    needs_review: true,
                    answers: vec![
                        GradedAnswer::Wrong {
                            group_id: "q1".into(),
                            marked_indices: vec![1],
                            correct_indices: vec![0],
                        },
                        GradedAnswer::Uncertain {
                            group_id: "q2".into(),
                            uncertain_indices: vec![2],
                        },
                    ],
                },
            ],
        }
    }

    /// `.xlsx` files are zip archives; the local-file-header magic is `PK\x03\x04`.
    fn is_xlsx(buf: &[u8]) -> bool {
        buf.len() > 4 && &buf[0..4] == b"PK\x03\x04"
    }

    #[test]
    fn build_report_produces_a_workbook() {
        let buf = build_report(&sample(), &labels()).unwrap();
        assert!(is_xlsx(&buf), "output must be a zip-based xlsx");
        assert!(buf.len() > 500, "a three-sheet workbook is non-trivial");
    }

    #[test]
    fn build_report_handles_empty_rows() {
        let report = XlsxReport {
            title: "empty".into(),
            questions: vec![QuestionColumn {
                group_id: "q1".into(),
                label: "1".into(),
            }],
            rows: vec![],
        };
        let buf = build_report(&report, &labels()).unwrap();
        assert!(is_xlsx(&buf));
    }

    #[test]
    fn build_report_handles_no_questions_no_rows() {
        let report = XlsxReport {
            title: "empty".into(),
            questions: vec![],
            rows: vec![],
        };
        let buf = build_report(&report, &labels()).unwrap();
        assert!(is_xlsx(&buf));
    }
}
