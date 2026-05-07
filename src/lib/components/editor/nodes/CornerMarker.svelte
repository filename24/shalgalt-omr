<script lang="ts">
  import { Circle } from "svelte-konva";
  import type { Marker } from "$lib/types/template";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import {
    clampNorm,
    toCanvasPx,
    toNormalized,
    type BackdropRect,
  } from "$lib/components/editor/coords";

  interface Props {
    marker: Marker;
    index: 0 | 1 | 2 | 3;
    rect: BackdropRect;
  }

  let { marker, index, rect }: Props = $props();

  // Stable color per corner so misordering is visually obvious. Order matches
  // OmrTemplate.markers: TL / TR / BR / BL.
  const colors = ["#ef4444", "#22c55e", "#3b82f6", "#eab308"] as const;

  // Marker handle radius in stage pixels. Independent of `marker.size` (which
  // is the printed marker dimension on the page) — this is just the editor
  // affordance.
  const HANDLE_RADIUS = 10;

  const px = $derived(toCanvasPx(marker.position, rect));

  function handleDragMove(e: { target: { x: () => number; y: () => number } }) {
    if (rect.width <= 0 || rect.height <= 0) return;
    const norm = clampNorm(toNormalized({ x: e.target.x(), y: e.target.y() }, rect));
    if (currentTemplate.draft) {
      currentTemplate.draft.markers[index].position = norm;
    }
  }
</script>

<Circle
  x={px.x}
  y={px.y}
  radius={HANDLE_RADIUS}
  fill={colors[index]}
  stroke="white"
  strokeWidth={2}
  draggable
  ondragmove={handleDragMove}
  ondragend={handleDragMove}
/>
