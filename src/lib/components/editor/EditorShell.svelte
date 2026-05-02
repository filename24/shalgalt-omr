<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import Toolbar from "$lib/components/editor/Toolbar.svelte";
  import LayerTree from "$lib/components/editor/LayerTree.svelte";
  import Inspector from "$lib/components/editor/Inspector.svelte";
  import TemplateCanvas from "$lib/components/editor/TemplateCanvas.svelte";

  let canvasWrapper = $state<HTMLDivElement | null>(null);
  let canvasWidth = $state(0);
  let canvasHeight = $state(0);

  // ResizeObserver-driven width/height so the Konva stage tracks pane resizes.
  $effect(() => {
    if (!canvasWrapper) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        canvasWidth = Math.max(0, entry.contentRect.width);
        canvasHeight = Math.max(0, entry.contentRect.height);
      }
    });
    ro.observe(canvasWrapper);
    return () => ro.disconnect();
  });

  // The empty-canvas CTA mirrors the toolbar's "Import Backdrop > Image" path.
  // Routing the click through the DOM is fragile, so the canvas only renders
  // the button label and the toolbar owns the import flow itself.
</script>

<div class="bg-background flex h-full min-h-0 flex-col">
  <Toolbar />
  <div class="flex-1 min-h-0">
    <PaneGroup direction="horizontal" autoSaveId="editor-shell">
      <Pane defaultSize={20} minSize={12} class="border-r">
        <LayerTree />
      </Pane>
      <PaneResizer class="bg-border hover:bg-accent w-1 cursor-col-resize" />
      <Pane defaultSize={55} minSize={30}>
        <div bind:this={canvasWrapper} class="bg-muted/30 h-full w-full">
          <TemplateCanvas width={canvasWidth} height={canvasHeight} />
        </div>
      </Pane>
      <PaneResizer class="bg-border hover:bg-accent w-1 cursor-col-resize" />
      <Pane defaultSize={25} minSize={18} class="border-l">
        <Inspector />
      </Pane>
    </PaneGroup>
  </div>
</div>
