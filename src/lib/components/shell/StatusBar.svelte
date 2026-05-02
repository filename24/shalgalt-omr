<script lang="ts">
  import { progress } from "$lib/stores/progress.svelte";
  import { session } from "$lib/stores/session.svelte";

  // P0 placeholder labels. Replaced by Mongolian copy from the P1 string-table.
  const stageLabel: Record<string, string> = {
    loading_pdf: "Loading PDF",
    rasterizing: "Rasterizing",
    detecting_markers: "Detecting markers",
    reading_bubbles: "Reading bubbles",
    grading: "Grading",
    saving: "Saving",
    done: "Done",
    failed: "Failed",
  };

  const last = $derived(progress.last);
  const pct = $derived(
    last && last.total > 0 ? Math.round((last.processed / last.total) * 100) : null,
  );
  const isActive = $derived(
    last !== null && last.stage !== "done" && last.stage !== "failed",
  );
</script>

<footer
  class="bg-sidebar text-muted-foreground flex h-7 shrink-0 items-center gap-3 border-t px-3 text-xs"
>
  <span class="flex items-center gap-1.5">
    <span
      class="size-1.5 rounded-full"
      class:bg-primary={isActive}
      class:bg-muted-foreground={!isActive}
    ></span>
    {isActive ? "Working" : "Idle"}
  </span>

  {#if last}
    <span class="border-border border-l pl-3">
      {stageLabel[last.stage] ?? last.stage}
      {#if pct !== null}
        — {pct}% ({last.processed}/{last.total})
      {/if}
    </span>
  {/if}

  <span class="ml-auto flex items-center gap-3">
    {#if session.activeJobId}
      <span>job <code class="font-mono">{session.activeJobId.slice(0, 8)}</code></span>
    {/if}
    {#if session.lastOpenedTemplateId !== null}
      <span>tmpl #{session.lastOpenedTemplateId}</span>
    {/if}
  </span>
</footer>
