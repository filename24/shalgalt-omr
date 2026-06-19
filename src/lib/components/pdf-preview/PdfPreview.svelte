<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import RefreshCcwIcon from "@lucide/svelte/icons/refresh-ccw";

  import { mn } from "$lib/i18n";
  import { renderPdfPreview } from "$lib/ipc/pdf";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";

  type PreviewState = "idle" | "loading" | "ready" | "error";

  let status = $state<PreviewState>("idle");
  let pngPath = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | null = null;
  // Bumped via the Refresh button to force a re-render even when the template
  // dependency snapshot has not changed (e.g. preview pane was just opened).
  let manualNonce = $state(0);

  // Cheap dependency snapshot: re-render when the JSON shape changes. Reading
  // the stringified template inside the effect makes Svelte 5's $effect track
  // every nested field.
  const draftKey = $derived(
    currentTemplate.draft ? JSON.stringify(currentTemplate.draft) : null,
  );

  $effect(() => {
    void manualNonce;
    const key = draftKey;
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (!key || !currentTemplate.draft) {
      status = "idle";
      pngPath = null;
      return;
    }
    status = "loading";
    const snapshot = currentTemplate.draft;
    timer = setTimeout(async () => {
      try {
        const path = await renderPdfPreview({ template: snapshot });
        pngPath = path;
        status = "ready";
      } catch (e) {
        console.error("preview render failed", e);
        status = "error";
      }
    }, 300);
  });

  // Cache-busting query string forces the browser to reload after a
  // re-render that produced a new PNG at the same asset path.
  const assetSrc = $derived(
    pngPath ? `${convertFileSrc(pngPath)}?t=${pngPath.length}-${manualNonce}` : null,
  );

  function refresh() {
    if (!currentTemplate.draft) {
      toast.error(mn.editor.preview.empty);
      return;
    }
    manualNonce += 1;
  }
</script>

<div class="flex h-full min-h-0 flex-col bg-background">
  <div class="flex h-10 items-center justify-between border-b px-3">
    <div class="flex flex-col">
      <span class="text-xs font-medium">{mn.editor.preview.title}</span>
      <span class="text-muted-foreground text-[10px]">{mn.editor.preview.paperHint}</span>
    </div>
    <Button
      variant="ghost"
      size="sm"
      onclick={refresh}
      disabled={!currentTemplate.draft || status === "loading"}
      title={mn.editor.toolbar.refreshPreview}
    >
      <RefreshCcwIcon class="size-4" />
    </Button>
  </div>

  <div class="flex-1 min-h-0 overflow-auto bg-muted/20 p-3">
    {#if status === "idle"}
      <div class="text-muted-foreground flex h-full items-center justify-center text-xs">
        {mn.editor.preview.empty}
      </div>
    {:else if status === "loading"}
      <Skeleton class="aspect-[210/297] w-full" />
    {:else if status === "error"}
      <div class="text-destructive flex h-full items-center justify-center text-xs">
        {mn.editor.preview.failed}
      </div>
    {:else if assetSrc}
      <img
        src={assetSrc}
        alt={mn.editor.preview.title}
        loading="lazy"
        decoding="async"
        class="mx-auto max-w-full bg-white shadow-sm"
      />
    {/if}
  </div>
</div>
