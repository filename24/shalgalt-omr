<script lang="ts">
  import { Layer } from "svelte-konva";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import CornerMarker from "$lib/components/editor/nodes/CornerMarker.svelte";
  import type { BackdropRect } from "$lib/components/editor/coords";

  interface Props {
    rect: BackdropRect;
  }

  let { rect }: Props = $props();

  // The 4-tuple is fixed by `OmrTemplate.markers`. `index` is typed as `0|1|2|3`
  // so `CornerMarker` can pick the right color and write back to the right slot.
  const indices = [0, 1, 2, 3] as const;
</script>

<Layer>
  {#if currentTemplate.draft}
    {#each indices as i (i)}
      <CornerMarker marker={currentTemplate.draft.markers[i]} index={i} {rect} />
    {/each}
  {/if}
</Layer>
