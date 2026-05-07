import type { TemplatePoint } from "$lib/types/template";

/**
 * Editor-only convenience descriptor for a 1-D bubble group. NEVER serialized
 * (Rule 3 mandates raw `bubbles: TemplatePoint[]` is the persisted form). The
 * editor expands a layout into bubble points whenever the user changes count,
 * direction, or spacing, and on reopen attempts to recover a layout via
 * `infer` so the user gets the structured controls back.
 */
export interface GroupLayout {
  origin: TemplatePoint;
  direction: "horizontal" | "vertical";
  spacing: number;
  count: number;
}

/** Expand a layout into the array of bubble points stored on the group. */
export function expand(layout: GroupLayout): TemplatePoint[] {
  const dx = layout.direction === "horizontal" ? layout.spacing : 0;
  const dy = layout.direction === "vertical" ? layout.spacing : 0;
  const out: TemplatePoint[] = [];
  for (let i = 0; i < layout.count; i++) {
    out.push({
      x: layout.origin.x + dx * i,
      y: layout.origin.y + dy * i,
    });
  }
  return out;
}

const EPSILON = 1e-4;

/**
 * Best-effort recovery of a `GroupLayout` from a saved bubbles array.
 * Returns `null` when the bubbles are not on a clean horizontal or vertical
 * line — the editor falls back to read-only "manual layout" rendering in that
 * case.
 */
export function infer(bubbles: TemplatePoint[]): GroupLayout | null {
  if (bubbles.length === 0) return null;
  if (bubbles.length === 1) {
    return {
      origin: { ...bubbles[0]! },
      direction: "horizontal",
      spacing: 0.05,
      count: 1,
    };
  }

  const first = bubbles[0]!;
  const second = bubbles[1]!;
  const dxSeed = second.x - first.x;
  const dySeed = second.y - first.y;
  const horizontalDom = Math.abs(dxSeed) >= Math.abs(dySeed);
  const direction: GroupLayout["direction"] = horizontalDom ? "horizontal" : "vertical";
  const spacing = horizontalDom ? dxSeed : dySeed;
  if (Math.abs(spacing) < EPSILON) return null;

  for (let i = 1; i < bubbles.length; i++) {
    const expected = {
      x: first.x + (horizontalDom ? spacing : 0) * i,
      y: first.y + (horizontalDom ? 0 : spacing) * i,
    };
    const got = bubbles[i]!;
    if (Math.abs(got.x - expected.x) > EPSILON || Math.abs(got.y - expected.y) > EPSILON) {
      return null;
    }
  }

  return {
    origin: { ...first },
    direction,
    spacing,
    count: bubbles.length,
  };
}
