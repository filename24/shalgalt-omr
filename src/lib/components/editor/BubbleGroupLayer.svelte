<script lang="ts">
  import { Layer } from "svelte-konva";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { editorSelection } from "$lib/stores/editorSelection.svelte";
  import BubbleGroupNode from "$lib/components/editor/nodes/BubbleGroupNode.svelte";
  import type { BackdropRect } from "$lib/components/editor/coords";

  interface Props {
    rect: BackdropRect;
  }

  let { rect }: Props = $props();

  function handleLayerClick(e: { target: { getStage?: () => unknown } }) {
    // Click on the empty layer (target is Stage itself) clears selection.
    if (typeof e.target.getStage !== "function") return;
    if (e.target.getStage() === e.target) {
      editorSelection.clear();
    }
  }
</script>

<Layer onclick={handleLayerClick} ontap={handleLayerClick}>
  {#if currentTemplate.draft}
    {#each currentTemplate.draft.groups as group (group.id)}
      <BubbleGroupNode {group} {rect} />
    {/each}
  {/if}
</Layer>
