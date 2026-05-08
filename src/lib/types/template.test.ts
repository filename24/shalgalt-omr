import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  createEmptyTemplate,
  templateSchema,
  TEMPLATE_VERSION,
  type OmrTemplate,
} from "./template";

// Fixture moved to `crates/shalgalt-core/tests/fixtures/` in P2-02 when the
// domain types were extracted into the shalgalt-core crate.
const FIXTURE_PATH = resolve(
  __dirname,
  "../../../crates/shalgalt-core/tests/fixtures/template-v1.json",
);

describe("OmrTemplate round-trip parity", () => {
  test("createEmptyTemplate output is schema-valid", () => {
    const t = createEmptyTemplate({ title: "test" });
    expect(() => templateSchema.parse(t)).not.toThrow();
    expect(t.version).toBe(TEMPLATE_VERSION);
    expect(t.markers).toHaveLength(4);
    expect(t.groups).toHaveLength(0);
  });

  test("schema parse round-trips an in-memory fixture", () => {
    const fixture: OmrTemplate = {
      version: 1,
      title: "round-trip",
      markers: [
        { id: "m-tl", position: { x: 0.05, y: 0.05 }, size: 0.02 },
        { id: "m-tr", position: { x: 0.95, y: 0.05 }, size: 0.02 },
        { id: "m-br", position: { x: 0.95, y: 0.95 }, size: 0.02 },
        { id: "m-bl", position: { x: 0.05, y: 0.95 }, size: 0.02 },
      ],
      groups: [
        {
          id: "grp-sid",
          kind: "student_id",
          label: "sid",
          bubbles: [
            { x: 0.1, y: 0.2 },
            { x: 0.15, y: 0.2 },
          ],
          answer_index: null,
          score: 1,
        },
        {
          id: "grp-q1",
          kind: "question",
          label: "Q1",
          bubbles: [
            { x: 0.3, y: 0.4 },
            { x: 0.35, y: 0.4 },
            { x: 0.4, y: 0.4 },
          ],
          answer_index: 2,
          score: 2.5,
        },
      ],
    };
    const serialized = JSON.stringify(fixture);
    const reparsed = templateSchema.parse(JSON.parse(serialized));
    expect(reparsed).toEqual(fixture);
  });

  test("Rust-generated fixture is accepted by the TS Zod schema", () => {
    // This guards field-name parity between `crates/shalgalt-core/src/domain/template.rs`
    // (serde) and `src/lib/types/template.ts` (Zod). If a Rust-side field
    // rename or type change ships without updating Zod, this test fails.
    const text = readFileSync(FIXTURE_PATH, "utf8");
    const parsed = templateSchema.parse(JSON.parse(text));
    expect(parsed.version).toBe(1);
    expect(parsed.markers).toHaveLength(4);
    expect(parsed.groups[0]?.kind).toBe("student_id");
    expect(parsed.groups[1]?.kind).toBe("question");
    expect(parsed.groups[1]?.answer_index).toBe(2);
  });
});

describe("OmrTemplate validation", () => {
  test("rejects fewer than 4 markers", () => {
    const bad = createEmptyTemplate();
    bad.markers = bad.markers.slice(0, 3) as unknown as typeof bad.markers;
    expect(() => templateSchema.parse(bad)).toThrow();
  });

  test("rejects negative score", () => {
    const t = createEmptyTemplate();
    t.groups.push({
      id: "g",
      kind: "question",
      label: "Q",
      bubbles: [],
      answer_index: null,
      score: -1,
    });
    expect(() => templateSchema.parse(t)).toThrow();
  });

  test("rejects negative answer_index", () => {
    const t = createEmptyTemplate();
    t.groups.push({
      id: "g",
      kind: "question",
      label: "Q",
      bubbles: [],
      answer_index: -1,
      score: 1,
    });
    expect(() => templateSchema.parse(t)).toThrow();
  });

  test("rejects unknown bubble kind", () => {
    const bad = {
      version: 1,
      title: "t",
      markers: [
        { id: "m1", position: { x: 0, y: 0 }, size: 0.02 },
        { id: "m2", position: { x: 1, y: 0 }, size: 0.02 },
        { id: "m3", position: { x: 1, y: 1 }, size: 0.02 },
        { id: "m4", position: { x: 0, y: 1 }, size: 0.02 },
      ],
      groups: [
        {
          id: "g",
          kind: "essay",
          label: "essay",
          bubbles: [],
          answer_index: null,
          score: 1,
        },
      ],
    };
    expect(() => templateSchema.parse(bad)).toThrow();
  });
});
