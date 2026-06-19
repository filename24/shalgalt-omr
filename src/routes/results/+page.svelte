<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import { mn } from "$lib/i18n";
  import { listTemplates } from "$lib/db/templates";
  import {
    listResultJobs,
    countResultJobs,
    type ResultJobRow,
    type ResultJobFilter,
  } from "$lib/db/jobs";
  import { exportJobById } from "$lib/results/export";
  import type { TemplateSummary } from "$lib/types/template";

  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";

  const PAGE_SIZE = 20;

  let templates = $state<TemplateSummary[]>([]);
  let jobs = $state<ResultJobRow[] | null>(null);
  let total = $state(0);
  let templateFilter = $state<number | null>(null);
  let needsReviewOnly = $state(false);
  let page = $state(1);
  let exportingId = $state<number | null>(null);

  const totalPages = $derived(Math.max(1, Math.ceil(total / PAGE_SIZE)));

  onMount(() => {
    void init();
  });

  async function init(): Promise<void> {
    try {
      templates = await listTemplates();
    } catch (e) {
      toast.error(mn.results.loadFailed, { description: String(e) });
    }
    await load();
  }

  function currentFilter(): ResultJobFilter {
    return {
      templateId: templateFilter,
      needsReviewOnly,
      limit: PAGE_SIZE,
      offset: (page - 1) * PAGE_SIZE,
    };
  }

  async function load(): Promise<void> {
    try {
      const filter = currentFilter();
      const [rows, count] = await Promise.all([
        listResultJobs(filter),
        countResultJobs(filter),
      ]);
      jobs = rows;
      total = count;
    } catch (e) {
      toast.error(mn.results.loadFailed, { description: String(e) });
      jobs = [];
    }
  }

  async function applyFilters(): Promise<void> {
    page = 1;
    await load();
  }

  async function goToPage(next: number): Promise<void> {
    page = Math.min(Math.max(1, next), totalPages);
    await load();
  }

  async function exportRow(id: number): Promise<void> {
    exportingId = id;
    try {
      await exportJobById(id);
    } catch {
      // exportJobById already surfaced the error via toast.
    } finally {
      exportingId = null;
    }
  }

  function formatDate(value: string): string {
    // SQLite CURRENT_TIMESTAMP is "YYYY-MM-DD HH:MM:SS" — show the date part.
    return value.slice(0, 10);
  }

  function formatPagination(): string {
    return mn.results.pagination.info
      .replace("{page}", String(page))
      .replace("{total}", String(totalPages));
  }
</script>

<section class="flex h-full flex-col gap-6 p-8">
  <header class="flex items-center gap-3">
    <h2 class="text-2xl font-bold">{mn.results.title}</h2>
    <p class="text-muted-foreground text-sm">{mn.results.subtitle}</p>
  </header>

  <div class="flex flex-wrap items-center gap-4">
    <label class="flex items-center gap-2 text-sm">
      <span class="text-muted-foreground">{mn.results.filters.template}</span>
      <select
        class="border-input bg-background h-9 rounded-md border px-3 text-sm"
        bind:value={templateFilter}
        onchange={applyFilters}
      >
        <option value={null}>{mn.results.filters.allTemplates}</option>
        {#each templates as t (t.id)}
          <option value={t.id}>{t.title}</option>
        {/each}
      </select>
    </label>

    <label class="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        class="size-4"
        bind:checked={needsReviewOnly}
        onchange={applyFilters}
      />
      <span>{mn.results.filters.needsReviewOnly}</span>
    </label>
  </div>

  <Card.Root class="flex-1 overflow-hidden">
    <Card.Content class="p-0">
      {#if jobs === null}
        <p class="text-muted-foreground p-6 text-sm">{mn.results.loading}</p>
      {:else if jobs.length === 0}
        <p class="text-muted-foreground p-6 text-sm">{mn.results.empty}</p>
      {:else}
        <table class="w-full text-sm">
          <thead class="text-muted-foreground border-b text-left">
            <tr>
              <th class="px-4 py-3 font-medium">{mn.results.table.template}</th>
              <th class="px-4 py-3 font-medium">{mn.results.table.date}</th>
              <th class="px-4 py-3 text-right font-medium">
                {mn.results.table.sheets}
              </th>
              <th class="px-4 py-3 font-medium">{mn.results.table.needsReview}</th>
              <th class="px-4 py-3 text-right font-medium">
                {mn.results.table.actions}
              </th>
            </tr>
          </thead>
          <tbody>
            {#each jobs as job (job.id)}
              <tr class="hover:bg-muted/40 border-b last:border-0">
                <td class="px-4 py-3 font-medium">{job.template_title}</td>
                <td class="text-muted-foreground px-4 py-3">
                  {formatDate(job.created_at)}
                </td>
                <td class="px-4 py-3 text-right tabular-nums">
                  {job.processed_pages}
                </td>
                <td class="px-4 py-3">
                  {#if job.needs_review_count > 0}
                    <Badge variant="destructive">
                      {job.needs_review_count}
                    </Badge>
                  {:else}
                    <Badge variant="secondary">{mn.results.badge.ready}</Badge>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  <div class="flex items-center justify-end gap-2">
                    <Button
                      size="sm"
                      variant="ghost"
                      onclick={() => goto(`/results/${job.id}`)}
                    >
                      <EyeIcon />
                      {mn.results.table.view}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={exportingId === job.id}
                      onclick={() => exportRow(job.id)}
                    >
                      <FileSpreadsheetIcon />
                      {mn.results.table.export}
                    </Button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </Card.Content>
  </Card.Root>

  {#if jobs && total > PAGE_SIZE}
    <footer class="flex items-center justify-end gap-3 text-sm">
      <Button
        size="sm"
        variant="outline"
        disabled={page <= 1}
        onclick={() => goToPage(page - 1)}
      >
        {mn.results.pagination.prev}
      </Button>
      <span class="text-muted-foreground tabular-nums">{formatPagination()}</span>
      <Button
        size="sm"
        variant="outline"
        disabled={page >= totalPages}
        onclick={() => goToPage(page + 1)}
      >
        {mn.results.pagination.next}
      </Button>
    </footer>
  {/if}
</section>
