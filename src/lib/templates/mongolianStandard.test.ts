import { describe, expect, test } from "vitest";
import { templateSchema } from "$lib/types/template";
import {
  createMongolianStandardTemplate,
  STANDARD_SECTIONS,
} from "./mongolianStandard";

describe("createMongolianStandardTemplate", () => {
  test("produces 51 groups in the expected sectional layout", () => {
    const t = createMongolianStandardTemplate();
    expect(t.groups).toHaveLength(51);

    const bySection = new Map<string, number>();
    for (const g of t.groups) {
      const k = g.section ?? "—";
      bySection.set(k, (bySection.get(k) ?? 0) + 1);
    }

    expect(bySection.get(STANDARD_SECTIONS.shifr)).toBe(4);
    expect(bySection.get(STANDARD_SECTIONS.variant)).toBe(1);
    expect(bySection.get(STANDARD_SECTIONS.section1)).toBe(30);
    expect(bySection.get(STANDARD_SECTIONS.section21)).toBe(8);
    expect(bySection.get(STANDARD_SECTIONS.section22)).toBe(8);
  });

  test("output passes the strict templateSchema validation", () => {
    const t = createMongolianStandardTemplate({ title: "test" });
    expect(() => templateSchema.parse(t)).not.toThrow();
  });

  test("every coordinate stays within [0, 1]", () => {
    const t = createMongolianStandardTemplate();
    for (const m of t.markers) {
      expect(m.position.x).toBeGreaterThanOrEqual(0);
      expect(m.position.x).toBeLessThanOrEqual(1);
      expect(m.position.y).toBeGreaterThanOrEqual(0);
      expect(m.position.y).toBeLessThanOrEqual(1);
    }
    for (const g of t.groups) {
      for (const b of g.bubbles) {
        expect(b.x).toBeGreaterThanOrEqual(0);
        expect(b.x).toBeLessThanOrEqual(1);
        expect(b.y).toBeGreaterThanOrEqual(0);
        expect(b.y).toBeLessThanOrEqual(1);
      }
    }
  });

  test("Section 1 questions have 4 bubbles each (A/B/C/D options)", () => {
    const t = createMongolianStandardTemplate();
    const sec1 = t.groups.filter((g) => g.section === STANDARD_SECTIONS.section1);
    expect(sec1).toHaveLength(30);
    for (const g of sec1) {
      expect(g.bubbles).toHaveLength(4);
      expect(g.kind).toBe("question");
    }
  });

  test("cipher rows are student_id with 10 digit bubbles each", () => {
    const t = createMongolianStandardTemplate();
    const shifr = t.groups.filter((g) => g.section === STANDARD_SECTIONS.shifr);
    expect(shifr).toHaveLength(4);
    for (const g of shifr) {
      expect(g.bubbles).toHaveLength(10);
      expect(g.kind).toBe("student_id");
    }
  });

  test("Section 2 numeric blocks have 8 rows of 10 digit bubbles", () => {
    const t = createMongolianStandardTemplate();
    for (const section of [STANDARD_SECTIONS.section21, STANDARD_SECTIONS.section22]) {
      const rows = t.groups.filter((g) => g.section === section);
      expect(rows, section).toHaveLength(8);
      for (const g of rows) {
        expect(g.bubbles, section).toHaveLength(10);
      }
    }
  });
});
