/**
 * Unit tests for the `.shalgalt` DB round-trip glue (P4-04).
 *
 * The `$lib/db/*` repositories and `@tauri-apps/plugin-fs` are mocked so the
 * tests exercise only the assemble/restore mapping logic — no SQLite or Tauri
 * runtime. Covers the exam + answer-key round-trip: assembleProject bundling
 * `answer-keys.json` and setting `exam_id`, restoreProject recreating the
 * exam + answer keys with their `exam_id` remapped onto the new exam row, and
 * the `manifest.encrypted` flag derived purely from passphrase presence.
 */
import { beforeEach, describe, expect, test, vi } from "vitest";

import { createEmptyTemplate } from "$lib/types/template";
import type { AnswerKeyRecord } from "$lib/types/exam";
import type { Exam } from "$lib/types/exam";
import type { OpenedProject } from "$lib/ipc/project";

const getTemplate = vi.fn();
const createTemplate = vi.fn();
const getJobById = vi.fn();
const listAnswerKeysByExam = vi.fn();
const createAnswerKey = vi.fn();
const createExam = vi.fn();
const readTextFile = vi.fn();

vi.mock("$lib/db/templates", () => ({
  getTemplate: (...args: unknown[]) => getTemplate(...args),
  createTemplate: (...args: unknown[]) => createTemplate(...args),
}));

vi.mock("$lib/db/jobs", () => ({
  getJobById: (...args: unknown[]) => getJobById(...args),
  // parseGradedSheets is pure; keep the real-ish behavior for completeness.
  parseGradedSheets: (job: { graded_sheets_json: string | null }) =>
    job.graded_sheets_json ? JSON.parse(job.graded_sheets_json) : [],
}));

vi.mock("$lib/db/answerKeys", () => ({
  listAnswerKeysByExam: (...args: unknown[]) => listAnswerKeysByExam(...args),
  createAnswerKey: (...args: unknown[]) => createAnswerKey(...args),
}));

vi.mock("$lib/db/exams", () => ({
  createExam: (...args: unknown[]) => createExam(...args),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readTextFile: (...args: unknown[]) => readTextFile(...args),
}));

// Imported after the mocks are registered.
import { assembleProject, restoreProject } from "./index";

function record(
  id: number,
  examId: number,
  variant: string,
  groupIds: string[],
): AnswerKeyRecord {
  return {
    id,
    exam_id: examId,
    variant,
    answers: groupIds.map((group_id) => ({ group_id, correct_indices: [0] })),
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

function template(title: string) {
  return { id: 1, title, schema: createEmptyTemplate({ title }) };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("assembleProject", () => {
  test("bundles answer-keys.json and sets exam_id when examId given", async () => {
    getTemplate.mockResolvedValue(template("T"));
    listAnswerKeysByExam.mockResolvedValue([
      record(10, 5, "A", ["q1", "q2"]),
      record(11, 5, "B", ["q1", "q2"]),
    ]);

    const out = await assembleProject({
      templateId: 1,
      examId: 5,
      title: "T",
    });

    expect(listAnswerKeysByExam).toHaveBeenCalledWith(5);

    const keysEntry = out.entries.find((e) => e.name === "answer-keys.json");
    expect(keysEntry).toBeDefined();
    expect(keysEntry!.kind).toBe("inline");
    const keys = JSON.parse(
      (keysEntry as { content: string }).content,
    ) as Array<{ exam_id: number; variant: string }>;
    expect(keys.map((k) => k.variant)).toEqual(["A", "B"]);
    expect(keys.every((k) => k.exam_id === 5)).toBe(true);

    const manifest = JSON.parse(out.manifestJson) as { exam_id?: number };
    expect(manifest.exam_id).toBe(5);
  });

  test("omits exam_id and answer-keys.json for a template-only export", async () => {
    getTemplate.mockResolvedValue(template("T"));

    const out = await assembleProject({ templateId: 1, title: "T" });

    expect(listAnswerKeysByExam).not.toHaveBeenCalled();
    expect(out.entries.find((e) => e.name === "answer-keys.json")).toBeUndefined();
    const manifest = JSON.parse(out.manifestJson) as { exam_id?: number };
    expect(manifest.exam_id).toBeUndefined();
  });

  test("exam keys take precedence over the job's single key", async () => {
    getTemplate.mockResolvedValue(template("T"));
    listAnswerKeysByExam.mockResolvedValue([record(10, 5, "A", ["q1"])]);
    getJobById.mockResolvedValue({
      id: 7,
      pdf_path: "/scan.pdf",
      answer_key_json: JSON.stringify({
        exam_id: 99,
        variant: "Z",
        answers: [{ group_id: "q1", correct_indices: [0] }],
      }),
      graded_sheets_json: null,
    });

    const out = await assembleProject({
      templateId: 1,
      examId: 5,
      jobId: 7,
      title: "T",
    });

    const keysEntries = out.entries.filter((e) => e.name === "answer-keys.json");
    expect(keysEntries).toHaveLength(1);
    const keys = JSON.parse(
      (keysEntries[0] as { content: string }).content,
    ) as Array<{ variant: string }>;
    expect(keys.map((k) => k.variant)).toEqual(["A"]);
  });
});

/**
 * The Rust writer rejects any archive where `manifest.encrypted` disagrees with
 * whether a passphrase was supplied, so the flag must be derived purely from the
 * passphrase. These tests lock that contract.
 */
describe("assembleProject — manifest.encrypted", () => {
  test("is false when no passphrase is given", async () => {
    getTemplate.mockResolvedValue(template("T"));
    const { manifestJson } = await assembleProject({ templateId: 1, title: "T" });
    expect(JSON.parse(manifestJson).encrypted).toBe(false);
  });

  test("is true when a passphrase is given", async () => {
    getTemplate.mockResolvedValue(template("T"));
    const { manifestJson } = await assembleProject({
      templateId: 1,
      title: "T",
      passphrase: "hunter2",
    });
    expect(JSON.parse(manifestJson).encrypted).toBe(true);
  });

  test("treats an empty passphrase as no encryption", async () => {
    getTemplate.mockResolvedValue(template("T"));
    const { manifestJson } = await assembleProject({
      templateId: 1,
      title: "T",
      passphrase: "",
    });
    expect(JSON.parse(manifestJson).encrypted).toBe(false);
  });
});

describe("restoreProject", () => {
  function opened(manifestTitle: string): OpenedProject {
    return {
      manifest: {
        format_version: 1,
        title: manifestTitle,
        created_at: "2026-01-01T00:00:00Z",
        sheet_count: 0,
        encrypted: false,
        exam_id: 5,
      },
      workspaceDir: "/ws",
      entries: [
        { name: "template.json", path: "/ws/template.json" },
        { name: "answer-keys.json", path: "/ws/answer-keys.json" },
      ],
    };
  }

  test("recreates exam + answer keys with remapped exam_id", async () => {
    const tpl = createEmptyTemplate({ title: "Imported" });
    const keysJson = JSON.stringify([
      {
        exam_id: 5,
        variant: "A",
        answers: [{ group_id: "q1", correct_indices: [0] }],
      },
      {
        exam_id: 5,
        variant: "B",
        answers: [{ group_id: "q1", correct_indices: [1] }],
      },
    ]);

    readTextFile.mockImplementation(async (path: string) =>
      path.endsWith("template.json") ? JSON.stringify(tpl) : keysJson,
    );
    createTemplate.mockResolvedValue(42);
    const exam: Exam = {
      id: 99,
      name: "My Exam",
      template_id: 42,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };
    createExam.mockResolvedValue(exam);
    createAnswerKey.mockResolvedValue(undefined);

    await restoreProject(opened("My Exam"));

    // Exam linked to the NEW template id, named from the manifest title.
    expect(createExam).toHaveBeenCalledWith({
      name: "My Exam",
      template_id: 42,
    });

    // Each key remapped onto the NEW exam id, variant preserved.
    expect(createAnswerKey).toHaveBeenCalledTimes(2);
    expect(createAnswerKey).toHaveBeenNthCalledWith(1, {
      exam_id: 99,
      variant: "A",
      answers: [{ group_id: "q1", correct_indices: [0] }],
    });
    expect(createAnswerKey).toHaveBeenNthCalledWith(2, {
      exam_id: 99,
      variant: "B",
      answers: [{ group_id: "q1", correct_indices: [1] }],
    });
  });

  test("normalizes a legacy single-object answer key into one exam key", async () => {
    const tpl = createEmptyTemplate({ title: "Imported" });
    const singleKeyJson = JSON.stringify({
      exam_id: 5,
      variant: "A",
      answers: [{ group_id: "q1", correct_indices: [0] }],
    });

    readTextFile.mockImplementation(async (path: string) =>
      path.endsWith("template.json") ? JSON.stringify(tpl) : singleKeyJson,
    );
    createTemplate.mockResolvedValue(42);
    createExam.mockResolvedValue({
      id: 100,
      name: "Legacy",
      template_id: 42,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    });
    createAnswerKey.mockResolvedValue(undefined);

    await restoreProject(opened("Legacy"));

    expect(createAnswerKey).toHaveBeenCalledTimes(1);
    expect(createAnswerKey).toHaveBeenCalledWith({
      exam_id: 100,
      variant: "A",
      answers: [{ group_id: "q1", correct_indices: [0] }],
    });
  });

  test("restores only the template when no answer keys are present", async () => {
    const tpl = createEmptyTemplate({ title: "Bare" });
    readTextFile.mockResolvedValue(JSON.stringify(tpl));
    createTemplate.mockResolvedValue(42);

    const bare: OpenedProject = {
      manifest: {
        format_version: 1,
        title: "Bare",
        created_at: "2026-01-01T00:00:00Z",
        sheet_count: 0,
        encrypted: false,
      },
      workspaceDir: "/ws",
      entries: [{ name: "template.json", path: "/ws/template.json" }],
    };

    await restoreProject(bare);

    expect(createTemplate).toHaveBeenCalledTimes(1);
    expect(createExam).not.toHaveBeenCalled();
    expect(createAnswerKey).not.toHaveBeenCalled();
  });
});
