import { describe, expect, test } from "vitest";

import {
  answerKeyMatchesTemplate,
  countMatchedQuestionGroups,
} from "./validateAnswerKey";
import type { BubbleGroup, OmrTemplate } from "$lib/types/template";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";

// Minimal fixtures — the validators only read `template.groups[].{id,kind}` and
// `answerKey.answers[].group_id`, so the rest of each shape is cast away.
function group(id: string, kind: BubbleGroup["kind"]): BubbleGroup {
  return { id, kind } as unknown as BubbleGroup;
}

function template(groups: BubbleGroup[]): OmrTemplate {
  return { groups } as unknown as OmrTemplate;
}

function answerKey(groupIds: string[]): AnswerKey {
  return {
    exam_id: 1,
    variant: "A",
    answers: groupIds.map((group_id) => ({ group_id, correct_indices: [0] })),
  };
}

describe("countMatchedQuestionGroups", () => {
  test("counts only question groups whose id appears in the key", () => {
    const tpl = template([
      group("sid", "student_id"),
      group("q-1", "question"),
      group("q-2", "question"),
      group("q-3", "question"),
    ]);
    const key = answerKey(["q-1", "q-3"]);

    expect(countMatchedQuestionGroups(tpl, key)).toBe(2);
  });

  test("ignores key entries that match a non-question group", () => {
    const tpl = template([
      group("sid", "student_id"),
      group("q-1", "question"),
    ]);
    // A stray entry pointing at the student-id group must not count.
    const key = answerKey(["sid"]);

    expect(countMatchedQuestionGroups(tpl, key)).toBe(0);
  });

  test("returns 0 when no key group_id matches any template group", () => {
    // The classic "template was edited / printed from a different template"
    // case: stale ids match nothing.
    const tpl = template([group("q-1", "question"), group("q-2", "question")]);
    const key = answerKey(["uuid-old-1", "uuid-old-2"]);

    expect(countMatchedQuestionGroups(tpl, key)).toBe(0);
  });
});

describe("answerKeyMatchesTemplate", () => {
  test("true when at least one question group is keyed", () => {
    const tpl = template([group("q-1", "question"), group("q-2", "question")]);
    expect(answerKeyMatchesTemplate(tpl, answerKey(["q-2"]))).toBe(true);
  });

  test("false when the key matches nothing (would yield empty results)", () => {
    const tpl = template([group("q-1", "question")]);
    expect(answerKeyMatchesTemplate(tpl, answerKey(["nope"]))).toBe(false);
  });

  test("false when the key is empty", () => {
    const tpl = template([group("q-1", "question")]);
    expect(answerKeyMatchesTemplate(tpl, answerKey([]))).toBe(false);
  });
});
