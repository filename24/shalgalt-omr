/**
 * Developer-facing grading diagnostics (console only).
 *
 * Reconstructs, from a single `task-result` payload, *why* a sheet was flagged
 * for manual review and *why* a value could not be recognized — then prints it
 * to the webview console so a teacher/dev can debug a misread scan without a
 * second pass. Everything here is gated behind {@link isGradeDebugEnabled}, so
 * it is silent in production builds unless explicitly switched on.
 *
 * This is intentionally English (Rule: developer tooling, not user-facing UI).
 */
import type { TaskResult } from "$lib/types/generated/TaskResult";
import type { GradedAnswer } from "$lib/types/generated/GradedAnswer";
import type { BubbleReading } from "$lib/types/generated/BubbleReading";

/**
 * The uncertain decision band (matches `BubbleReading` / ADR 0010): a bubble
 * whose fill lands in `[LOW, HIGH]` is ambiguous and forces `needs_review`.
 */
const UNCERTAIN_LOW = 0.35;
const UNCERTAIN_HIGH = 0.65;

/**
 * Whether to emit grading diagnostics. On automatically during `pnpm tauri dev`
 * (Vite dev build); otherwise opt in by running
 * `localStorage.setItem("shalgalt:debug", "1")` in the devtools console.
 */
export function isGradeDebugEnabled(): boolean {
  if (import.meta.env.DEV) return true;
  try {
    return (
      typeof localStorage !== "undefined" &&
      localStorage.getItem("shalgalt:debug") === "1"
    );
  } catch {
    return false;
  }
}

/** The discriminant key of a `GradedAnswer` union member, e.g. `"uncertain"`. */
function answerKind(a: GradedAnswer): string {
  return Object.keys(a)[0] ?? "unknown";
}

/** Every `GradedAnswer` payload carries a `group_id`; pull it out generically. */
function answerGroupId(a: GradedAnswer): string {
  const payload = Object.values(a)[0] as { group_id: string } | undefined;
  return payload?.group_id ?? "(unknown)";
}

/** Bubble indices the grader marked uncertain for this answer, or `[]`. */
function uncertainIndices(a: GradedAnswer): readonly number[] {
  return "uncertain" in a ? a.uncertain.uncertain_indices : [];
}

/** The measured fill for one bubble, or `undefined` when it was never sampled. */
function fillOf(
  readings: readonly BubbleReading[],
  groupId: string,
  index: number,
): number | undefined {
  return readings.find(
    (r) => r.group_id === groupId && r.bubble_index === index,
  )?.fill;
}

function fmtFill(fill: number | undefined): string {
  return fill === undefined ? "?" : fill.toFixed(2);
}

/**
 * Print a per-sheet diagnostic for one graded page. Explains the review reason
 * (unresolved variant / uncertain bubbles), flags un-decoded student id and
 * variant, summarizes the answer breakdown, and dumps every bubble's raw fill
 * so a misread is traceable to a specific fill value.
 */
export function logSheetDiagnostics(result: TaskResult): void {
  if (!isGradeDebugEnabled()) return;

  const { parsed, graded, page_index } = result;
  const studentId = graded.student_id_text ?? parsed.student_id_text;
  const verdict = graded.needs_review ? "⚠ NEEDS REVIEW" : "ok";

  console.groupCollapsed(
    `[grade] page ${page_index + 1} — ${verdict} — score ${graded.total_score.toFixed(1)}`,
  );

  console.log("student id:", studentId ?? "(not decoded)");
  console.log("variant:", parsed.variant ?? "(not decoded)");

  // --- Why a manual review is required ---------------------------------------
  if (graded.needs_review) {
    const reasons: string[] = [];

    // An unscored sheet (empty answers) means the variant never resolved to a key.
    if (graded.answers.length === 0) {
      reasons.push(
        parsed.variant
          ? `variant "${parsed.variant}" has no matching answer key — sheet left unscored`
          : "variant not decoded and no fallback key — sheet left unscored",
      );
    }

    for (const answer of graded.answers) {
      const idxs = uncertainIndices(answer);
      if (idxs.length === 0) continue;
      const gid = answerGroupId(answer);
      const detail = idxs
        .map((i) => `#${i}=${fmtFill(fillOf(parsed.readings, gid, i))}`)
        .join(", ");
      reasons.push(
        `uncertain "${gid}": fills [${detail}] in the ${UNCERTAIN_LOW}–${UNCERTAIN_HIGH} ambiguous band`,
      );
    }

    if (reasons.length > 0) {
      console.warn("needs review because:\n  - " + reasons.join("\n  - "));
    } else {
      console.warn("needs review (no specific reason reconstructed)");
    }
  }

  // --- Why a value could not be recognized -----------------------------------
  if (!studentId) {
    console.warn(
      "student id NOT decoded — marks too light/ambiguous, sheet blank, or template has no StudentId group",
    );
  }
  if (!parsed.variant) {
    console.warn(
      "variant NOT decoded — blank/ambiguous variant row, or template has no variant group (fallback used)",
    );
  }

  // --- Answer breakdown by verdict -------------------------------------------
  const breakdown = graded.answers.reduce<Record<string, number>>((acc, a) => {
    const kind = answerKind(a);
    acc[kind] = (acc[kind] ?? 0) + 1;
    return acc;
  }, {});
  console.log("answer breakdown:", breakdown);

  // --- Raw fill dump: see exactly why a bubble read filled / unfilled ---------
  console.groupCollapsed("bubble fills (group → index:fill)");
  const byGroup = new Map<string, BubbleReading[]>();
  for (const reading of parsed.readings) {
    const arr = byGroup.get(reading.group_id) ?? [];
    arr.push(reading);
    byGroup.set(reading.group_id, arr);
  }
  for (const [gid, arr] of byGroup) {
    const row = [...arr]
      .sort((a, b) => a.bubble_index - b.bubble_index)
      .map((r) => `${r.bubble_index}:${r.fill.toFixed(2)}`)
      .join("  ");
    console.log(`${gid}: ${row}`);
  }
  console.groupEnd();

  console.groupEnd();
}

/**
 * Print a job-level failure. Marker-detection failures abort before any
 * `task-result` is emitted, so this is the only place that recognition failure
 * surfaces; the message comes straight from the Rust pipeline.
 */
export function logJobFailure(message: string | null): void {
  if (!isGradeDebugEnabled()) return;
  console.error(`[grade] job FAILED: ${message ?? "unknown error"}`);
  console.error(
    "If this mentions ArUco markers, the four corner markers were not all detected — " +
      "check the scan/photo for cropping, glare, shadow, skew, or low resolution.",
  );
}
