import { describe, expect, test } from "vitest";

import { buildReportLabels, buildXlsxReport } from "./xlsxReport";
import type { OmrTemplate } from "$lib/types/template";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";
import type { GradedAnswer } from "$lib/types/generated/GradedAnswer";
import type { GradedJobSheet } from "$lib/types/job";

// Minimal fixtures — buildXlsxReport only reads `template.groups[].{id,label}`
// and the graded fields, so the rest of each shape is cast away for brevity.
function template(
  groups: { id: string; label: string; score?: number }[],
): OmrTemplate {
  return { groups } as unknown as OmrTemplate;
}

function answerKey(groupIds: string[]): AnswerKey {
  return {
    exam_id: 1,
    variant: "A",
    answers: groupIds.map((id) => ({ group_id: id, correct_indices: [0] })),
  };
}

function sheet(
  idText: string | undefined,
  score: number,
  needsReview: boolean,
  answers: GradedAnswer[],
  variant?: string,
): GradedJobSheet {
  return {
    page_index: 0,
    page_image_path: "/tmp/x.png",
    parsed: { student_id_text: idText, variant } as never,
    graded: {
      template_id: 1,
      student_id: null,
      student_id_text: idText,
      total_score: score,
      answers,
      needs_review: needsReview,
    },
    reviewed: false,
  };
}

describe("buildXlsxReport", () => {
  test("question columns follow the answer key order with template labels and scores", () => {
    const tpl = template([
      { id: "q1", label: "1-р асуулт", score: 2 },
      { id: "q2", label: "2-р асуулт", score: 3 },
      { id: "q3", label: "Түлхүүрт ороогүй", score: 1 },
    ]);
    // Key omits q3 → it must not become a column (answer key = question set).
    const report = buildXlsxReport({
      title: "Сорил",
      template: tpl,
      answerKey: answerKey(["q2", "q1"]),
      sheets: [],
      studentFallback: "Сурагч",
    });

    expect(report.questions).toEqual([
      { group_id: "q2", label: "2-р асуулт", score: 3 },
      { group_id: "q1", label: "1-р асуулт", score: 2 },
    ]);
  });

  test("falls back to group id and zero score when the template has no match", () => {
    const report = buildXlsxReport({
      title: "Сорил",
      template: template([]),
      answerKey: answerKey(["qX"]),
      sheets: [],
      studentFallback: "Сурагч",
    });
    expect(report.questions[0]).toEqual({ group_id: "qX", label: "qX", score: 0 });
  });

  test("rows carry score, review flag, and pass answers through unchanged", () => {
    const answers: GradedAnswer[] = [
      { correct: { group_id: "q1", marked_indices: [0] } },
    ];
    const report = buildXlsxReport({
      title: "Сорил",
      template: template([{ id: "q1", label: "1" }]),
      answerKey: answerKey(["q1"]),
      sheets: [sheet("00123", 1, false, answers)],
      studentFallback: "Сурагч",
    });

    expect(report.rows).toHaveLength(1);
    expect(report.rows[0]!.label).toBe("00123");
    expect(report.rows[0]!.total_score).toBe(1);
    expect(report.rows[0]!.needs_review).toBe(false);
    expect(report.rows[0]!.answers).toBe(answers);
  });

  test("carries the decoded variant onto the row, null when absent", () => {
    const report = buildXlsxReport({
      title: "Сорил",
      template: template([{ id: "q1", label: "1" }]),
      answerKey: answerKey(["q1"]),
      sheets: [
        sheet("00123", 1, false, [], "B"),
        sheet("00124", 1, false, []),
      ],
      studentFallback: "Сурагч",
    });

    expect(report.rows[0]!.variant).toBe("B");
    expect(report.rows[1]!.variant).toBeUndefined();
  });

  test("uses an indexed fallback label when no student id was read", () => {
    const report = buildXlsxReport({
      title: "Сорил",
      template: template([{ id: "q1", label: "1" }]),
      answerKey: answerKey(["q1"]),
      sheets: [
        sheet(undefined, 0, true, []),
        sheet("   ", 0, false, []),
      ],
      studentFallback: "Сурагч",
    });
    expect(report.rows[0]!.label).toBe("Сурагч 1");
    expect(report.rows[1]!.label).toBe("Сурагч 2");
    expect(report.rows[0]!.needs_review).toBe(true);
  });
});

describe("buildReportLabels", () => {
  test("maps every ReportLabels field to a non-empty string", () => {
    const labels = buildReportLabels();
    for (const [key, value] of Object.entries(labels)) {
      expect(value, key).toBeTypeOf("string");
      expect(value.length, key).toBeGreaterThan(0);
    }
  });
});
