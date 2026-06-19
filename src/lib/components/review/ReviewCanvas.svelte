<script lang="ts">
  /**
   * Manual-review canvas (P3-07).
   *
   * Renders a graded page raster with a Konva overlay of every bubble. Click
   * cycles a bubble reading through filled → unfilled → uncertain (in order to
   * accommodate ambiguous scans the CV pipeline could not classify). The
   * parent owns persistence — we just emit `onOverride(updatedReadings)`
   * whenever the user changes something. Re-grading is triggered by the
   * parent via `regrade_sheet`.
   *
   * Reuses `coords.ts` (`fitContain`, `toCanvasPx`) so the normalized
   * `[0,1]` template coordinates map to pixels the same way they do in the
   * editor — no duplicate math.
   */
  import { Stage, Layer, Image as KonvaImage, Circle } from "svelte-konva";
  import { fitContain, toCanvasPx } from "$lib/components/editor/coords";
  import { assetUrl } from "$lib/fs/templateAssets";
  import type { OmrTemplate } from "$lib/types/template";
  import type { BubbleReading } from "$lib/types/generated/BubbleReading";
  import type { GradedAnswer } from "$lib/types/generated/GradedAnswer";
  import type { GradedSheet } from "$lib/types/generated/GradedSheet";
  import type { AnswerKey } from "$lib/types/generated/AnswerKey";

  interface Props {
    width: number;
    height: number;
    template: OmrTemplate;
    pageImagePath: string;
    readings: BubbleReading[];
    graded: GradedSheet;
    /** The variant's key, or `null` when the sheet's variant is unresolved — the
     * canvas then renders marks without the correct-answer overlay. */
    answerKey: AnswerKey | null;
    onOverride: (next: BubbleReading[]) => void;
  }

  let {
    width,
    height,
    template,
    pageImagePath,
    readings,
    graded,
    answerKey,
    onOverride,
  }: Props = $props();

  // Reading values used when the user cycles a bubble. The CV pipeline
  // produces a continuous fill ratio; we map our cycle states to canonical
  // values inside each band so subsequent grading sees the same answer.
  const FILL_FILLED = 0.9;
  const FILL_UNFILLED = 0.05;
  const FILL_UNCERTAIN = 0.5;

  type FillBand = "filled" | "unfilled" | "uncertain";

  function bandOf(fill: number): FillBand {
    if (fill < 0.35) return "unfilled";
    if (fill > 0.65) return "filled";
    return "uncertain";
  }

  function nextBandFill(currentFill: number): number {
    switch (bandOf(currentFill)) {
      case "filled":
        return FILL_UNFILLED;
      case "unfilled":
        return FILL_UNCERTAIN;
      case "uncertain":
        return FILL_FILLED;
    }
  }

  let imgEl = $state<HTMLImageElement | null>(null);
  $effect(() => {
    const path = pageImagePath;
    if (!path) {
      imgEl = null;
      return;
    }
    const el = new Image();
    el.src = assetUrl(path);
    el.onload = () => {
      imgEl = el;
    };
    el.onerror = () => {
      imgEl = null;
    };
  });

  const rect = $derived(
    fitContain(
      width,
      height,
      imgEl?.naturalWidth ?? 0,
      imgEl?.naturalHeight ?? 0,
    ),
  );

  const BUBBLE_RADIUS = 9;

  function readingFor(groupId: string, idx: number): BubbleReading | undefined {
    return readings.find(
      (r) => r.group_id === groupId && r.bubble_index === idx,
    );
  }

  function correctIndicesFor(groupId: string): readonly number[] {
    return (
      answerKey?.answers.find((a) => a.group_id === groupId)?.correct_indices ??
      []
    );
  }

  function uncertainIndicesFor(groupId: string): readonly number[] {
    // Rust serializes `GradedAnswer` with `#[serde(rename_all = "snake_case")]`,
    // which ts-rs renders as `{ uncertain: { group_id, uncertain_indices } }`.
    for (const ans of graded.answers) {
      if ("uncertain" in ans) {
        const u = ans as Extract<GradedAnswer, { uncertain: unknown }>;
        if (u.uncertain.group_id === groupId) return u.uncertain.uncertain_indices;
      }
    }
    return [];
  }

  /**
   * Choose a fill colour for one bubble overlay. The semantics are
   * intentionally conservative — we colour by the *grading verdict* (correct /
   * wrong / uncertain / missed-correct) rather than by raw fill ratio so the
   * same overlay drives both the legend and the user's mental model.
   */
  function bubbleColors(
    groupId: string,
    idx: number,
  ): { fill: string; stroke: string } {
    const reading = readingFor(groupId, idx);
    const correctIdx = correctIndicesFor(groupId);
    const uncertainIdx = uncertainIndicesFor(groupId);
    const isCorrectAnswer = correctIdx.includes(idx);

    if (uncertainIdx.includes(idx)) {
      return { fill: "rgba(245,158,11,0.5)", stroke: "#b45309" };
    }
    if (!reading) {
      return { fill: "rgba(255,255,255,0.0)", stroke: "#9ca3af" };
    }
    const band = bandOf(reading.fill);
    if (band === "filled") {
      if (isCorrectAnswer) {
        return { fill: "rgba(22,163,74,0.55)", stroke: "#15803d" };
      }
      return { fill: "rgba(220,38,38,0.55)", stroke: "#b91c1c" };
    }
    if (band === "uncertain") {
      return { fill: "rgba(245,158,11,0.5)", stroke: "#b45309" };
    }
    // Unfilled.
    if (isCorrectAnswer) {
      return { fill: "rgba(255,255,255,0.0)", stroke: "#15803d" };
    }
    return { fill: "rgba(255,255,255,0.0)", stroke: "#9ca3af" };
  }

  function handleBubbleClick(
    groupId: string,
    idx: number,
    e: { cancelBubble: boolean },
  ): void {
    e.cancelBubble = true;
    const existing = readingFor(groupId, idx);
    const currentFill = existing?.fill ?? FILL_UNFILLED;
    const nextFill = nextBandFill(currentFill);

    let next: BubbleReading[];
    if (existing) {
      next = readings.map((r) =>
        r.group_id === groupId && r.bubble_index === idx
          ? { ...r, fill: nextFill, confidence: 1.0 }
          : r,
      );
    } else {
      next = [
        ...readings,
        {
          group_id: groupId,
          bubble_index: idx,
          fill: nextFill,
          confidence: 1.0,
        },
      ];
    }
    onOverride(next);
  }
</script>

<div class="bg-muted/30 relative h-full w-full overflow-hidden rounded-md border">
  {#if width > 0 && height > 0 && imgEl}
    <Stage {width} {height}>
      <Layer>
        <KonvaImage
          image={imgEl}
          x={rect.x}
          y={rect.y}
          width={rect.width}
          height={rect.height}
          listening={false}
        />
      </Layer>
      <Layer>
        {#each template.groups as group (group.id)}
          {#if group.kind === "question"}
            {#each group.bubbles as b, i (i)}
              {@const px = toCanvasPx(b, rect)}
              {@const colors = bubbleColors(group.id, i)}
              <Circle
                x={px.x}
                y={px.y}
                radius={BUBBLE_RADIUS}
                fill={colors.fill}
                stroke={colors.stroke}
                strokeWidth={2}
                onclick={(e) => handleBubbleClick(group.id, i, e)}
                ontap={(e) => handleBubbleClick(group.id, i, e)}
              />
            {/each}
          {/if}
        {/each}
      </Layer>
    </Stage>
  {:else}
    <div
      class="text-muted-foreground flex h-full w-full items-center justify-center text-sm"
    >
      …
    </div>
  {/if}
</div>
