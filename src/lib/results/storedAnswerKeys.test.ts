import { describe, expect, test } from "vitest";
import {
  parseStoredAnswerKeys,
  mergeAnswerKeysForColumns,
} from "./storedAnswerKeys";
import type { AnswerKey } from "$lib/types/generated/AnswerKey";

function key(variant: string, groupIds: string[]): AnswerKey {
  return {
    exam_id: 7,
    variant,
    answers: groupIds.map((group_id) => ({ group_id, correct_indices: [0] })),
  };
}

describe("parseStoredAnswerKeys", () => {
  test("wraps a legacy single-object payload in an array", () => {
    const json = JSON.stringify(key("A", ["q1", "q2"]));
    const out = parseStoredAnswerKeys(json);
    expect(out).toHaveLength(1);
    expect(out[0]!.variant).toBe("A");
  });

  test("returns the array form as-is", () => {
    const json = JSON.stringify([key("A", ["q1"]), key("B", ["q1"])]);
    const out = parseStoredAnswerKeys(json);
    expect(out.map((k) => k.variant)).toEqual(["A", "B"]);
  });
});

describe("mergeAnswerKeysForColumns", () => {
  test("unions question groups across variants in first-seen order", () => {
    const merged = mergeAnswerKeysForColumns([
      key("A", ["q1", "q2"]),
      key("B", ["q1", "q2", "q3"]),
    ]);
    expect(merged.answers.map((a) => a.group_id)).toEqual(["q1", "q2", "q3"]);
  });

  test("dedupes group ids shared across variants", () => {
    const merged = mergeAnswerKeysForColumns([
      key("A", ["q1", "q2"]),
      key("B", ["q2", "q1"]),
    ]);
    expect(merged.answers.map((a) => a.group_id)).toEqual(["q1", "q2"]);
  });

  test("yields an empty key for no variants", () => {
    const merged = mergeAnswerKeysForColumns([]);
    expect(merged.answers).toEqual([]);
  });
});
