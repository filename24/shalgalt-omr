<script lang="ts">
  import { Group, Circle } from "svelte-konva";
  import type { BubbleGroup } from "$lib/types/template";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { editorSelection } from "$lib/stores/editorSelection.svelte";
  import {
    clampNorm,
    toCanvasPx,
    toNormalized,
    type BackdropRect,
  } from "$lib/components/editor/coords";

  interface Props {
    group: BubbleGroup;
    rect: BackdropRect;
  }

  let { group, rect }: Props = $props();

  // Bubble handle radius in stage pixels. Independent of any printed bubble
  // size on the page — this is the editor affordance.
  const BUBBLE_RADIUS = 8;

  const isSelected = $derived(editorSelection.selectedGroupId === group.id);
  const isQuestion = $derived(group.kind === "question");

  // Pixel positions for every bubble in the group. The whole `<Group>` is
  // draggable and we apply the drag delta to every bubble in normalized space
  // on dragmove, so the children render at their per-bubble positions with
  // the Group's own offset at (0,0).
  const bubblesPx = $derived(group.bubbles.map((b) => toCanvasPx(b, rect)));

  function handleClickGroup(e: { cancelBubble: boolean }) {
    e.cancelBubble = true;
    editorSelection.select(group.id);
  }

  function handleClickBubble(idx: number, e: { cancelBubble: boolean }) {
    e.cancelBubble = true;
    editorSelection.select(group.id);
    if (!isQuestion || !currentTemplate.draft) return;
    const groupIdx = currentTemplate.draft.groups.findIndex((g) => g.id === group.id);
    if (groupIdx < 0) return;
    currentTemplate.draft.groups[groupIdx].answer_index = idx;
  }

  function handleGroupDragEnd(e: {
    target: {
      x: ((value?: number) => number) | (() => number);
      y: ((value?: number) => number) | (() => number);
    };
  }) {
    if (!currentTemplate.draft || rect.width <= 0 || rect.height <= 0) return;
    const groupIdx = currentTemplate.draft.groups.findIndex((g) => g.id === group.id);
    if (groupIdx < 0) return;

    const tx = (e.target.x as () => number)();
    const ty = (e.target.y as () => number)();
    const dx = tx / rect.width;
    const dy = ty / rect.height;

    // Reset the Konva node back to origin so the freshly-mutated bubble
    // coordinates aren't doubled by the dragged-group offset on next render.
    (e.target.x as (v: number) => number)(0);
    (e.target.y as (v: number) => number)(0);

    if (dx === 0 && dy === 0) return;

    const next = currentTemplate.draft.groups[groupIdx].bubbles.map((b) =>
      clampNorm({ x: b.x + dx, y: b.y + dy }),
    );
    currentTemplate.draft.groups[groupIdx].bubbles = next;
  }
</script>

<Group
  x={0}
  y={0}
  draggable
  ondragend={handleGroupDragEnd}
  onclick={handleClickGroup}
  ontap={handleClickGroup}
>
  {#each bubblesPx as p, i (i)}
    {@const isAnswer = isQuestion && group.answer_index === i}
    <Circle
      x={p.x}
      y={p.y}
      radius={BUBBLE_RADIUS}
      fill={isAnswer ? "#22c55e" : "rgba(255,255,255,0.4)"}
      stroke={isSelected ? "#3b82f6" : "#1f2937"}
      strokeWidth={isSelected ? 2.5 : 1.5}
      onclick={(e) => handleClickBubble(i, e)}
      ontap={(e) => handleClickBubble(i, e)}
    />
  {/each}
</Group>
