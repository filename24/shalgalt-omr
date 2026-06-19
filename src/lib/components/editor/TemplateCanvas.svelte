<script lang="ts">
  import { Stage, Layer, Image as KonvaImage, Rect } from "svelte-konva";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { mn } from "$lib/i18n";
  import {
    fitContainOrA4,
    type BackdropRect,
  } from "$lib/components/editor/coords";
  import { assetUrl } from "$lib/fs/templateAssets";
  import MarkerLayer from "$lib/components/editor/MarkerLayer.svelte";
  import BubbleGroupLayer from "$lib/components/editor/BubbleGroupLayer.svelte";

  interface Props {
    width: number;
    height: number;
  }

  let { width, height }: Props = $props();

  // Loaded HTMLImageElement for the backdrop. `null` when no backdrop is set
  // or while it is still loading.
  let imgEl = $state<HTMLImageElement | null>(null);

  // Reload the backdrop image whenever the path on the store changes. We
  // create a fresh `HTMLImageElement` rather than relying on Konva's path
  // loading so the natural width/height is available for letterboxing.
  $effect(() => {
    const path = currentTemplate.backdropPath;
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

  // Fall back to an A4-portrait rect when no image is loaded so the bubbles
  // still render meaningfully against a synthetic page (used by the
  // Mongolian-standard preset before the user imports a real backdrop).
  const rect: BackdropRect = $derived(
    fitContainOrA4(width, height, imgEl?.naturalWidth ?? 0, imgEl?.naturalHeight ?? 0),
  );

  const hasBackdrop = $derived(currentTemplate.backdropPath !== null && imgEl !== null);
</script>

{#if currentTemplate.draft && width > 0 && height > 0}
  <div class="relative h-full w-full">
    <Stage {width} {height}>
      <Layer>
        {#if hasBackdrop && imgEl}
          <KonvaImage
            image={imgEl}
            x={rect.x}
            y={rect.y}
            width={rect.width}
            height={rect.height}
            listening={false}
          />
        {:else}
          <Rect
            x={rect.x}
            y={rect.y}
            width={rect.width}
            height={rect.height}
            fill="white"
            stroke="#cbd5e1"
            strokeWidth={1}
            listening={false}
          />
        {/if}
      </Layer>
      <MarkerLayer {rect} />
      <BubbleGroupLayer {rect} />
    </Stage>

    {#if !hasBackdrop}
      <div
        class="text-muted-foreground pointer-events-none absolute top-2 left-1/2 -translate-x-1/2 rounded-full border bg-white/90 px-3 py-1 text-xs shadow-sm"
      >
        {mn.editor.empty.noBackdropHint}
      </div>
    {/if}
  </div>
{:else}
  <div
    class="text-muted-foreground flex h-full w-full items-center justify-center text-sm"
  >
    {mn.editor.empty.noBackdrop}
  </div>
{/if}
