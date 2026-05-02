<script lang="ts">
  import { listTemplates } from "$lib/db/templates";
  import type { TemplateSummary } from "$lib/types/template";

  let templates = $state<TemplateSummary[] | null>(null);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      templates = await listTemplates();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="p-8">
  <h2 class="mb-2 text-2xl font-bold">Dashboard</h2>
  <p class="mb-6 text-sm text-[var(--color-text-muted)]">
    Local-First OMR Grading IDE — Phase 0 foundation.
  </p>

  <button
    type="button"
    onclick={refresh}
    class="rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-black"
  >
    Refresh templates
  </button>

  {#if error}
    <p class="mt-4 text-sm text-red-400">{error}</p>
  {/if}

  {#if templates}
    <ul class="mt-4 space-y-2">
      {#each templates as t (t.id)}
        <li
          class="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] p-3"
        >
          <span class="font-medium">{t.title}</span>
          <span class="ml-2 text-xs text-[var(--color-text-muted)]">#{t.id}</span>
        </li>
      {:else}
        <li class="text-sm text-[var(--color-text-muted)]">
          No templates saved yet.
        </li>
      {/each}
    </ul>
  {/if}
</section>
