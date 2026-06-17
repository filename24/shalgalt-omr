<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";

  import { mn } from "$lib/i18n";
  import { getJobById, parseGradedSheets } from "$lib/db/jobs";
  import { getTemplate } from "$lib/db/templates";
  import { assetUrl } from "$lib/fs/templateAssets";
  import { exportJobToXlsx } from "$lib/results/export";
  import type { Job, GradedJobSheet } from "$lib/types/job";
  import type { OmrTemplate } from "$lib/types/template";
  import type { AnswerKey } from "$lib/types/generated/AnswerKey";
  import type { GradedAnswer } from "$lib/types/generated/GradedAnswer";

  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import PencilIcon from "@lucide/svelte/icons/pencil";

  let job = $state<Job | null>(null);
  let sheets = $state<GradedJobSheet[]>([]);
  let template = $state<OmrTemplate | null>(null);
  let answerKey = $state<AnswerKey | null>(null);
  let selectedIndex = $state(0);
  let loadFailed = $state<string | null>(null);
  let exporting = $state(false);

  const jobIdParam = $derived(Number(page.params.job_id));
  const selected = $derived(sheets[selectedIndex] ?? null);

  const labelByGroup = $derived(
    new Map((template?.groups ?? []).map((g) => [g.id, g.label])),
  );

  const avgScore = $derived(
    sheets.length === 0
      ? 0
      : sheets.reduce((sum, s) => sum + s.graded.total_score, 0) /
          sheets.length,
  );

  // Reload whenever the route id changes. Using `$effect` (not `onMount`) means
  // an in-app `goto('/results/N')` from this page reloads instead of showing
  // stale data; the reset clears the previous job while the new one loads.
  $effect(() => {
    const id = jobIdParam;
    loadFailed = null;
    job = null;
    template = null;
    selectedIndex = 0;
    void load(id);
  });

  async function load(id: number): Promise<void> {
    try {
      const j = await getJobById(id);
      if (!j) {
        loadFailed = mn.results.detail.notFound;
        return;
      }
      if (j.status !== "done") {
        loadFailed = mn.results.detail.notFinished;
        return;
      }
      job = j;
      sheets = parseGradedSheets(j);
      answerKey = JSON.parse(j.answer_key_json) as AnswerKey;
      const tpl = await getTemplate(j.template_id);
      if (tpl) template = tpl.schema;
    } catch (e) {
      loadFailed = String(e);
    }
  }

  /** External-tag of a graded answer: "correct" | "wrong" | "blank" | … */
  function outcomeKey(answer: GradedAnswer): string {
    return Object.keys(answer)[0] ?? "blank";
  }

  /** The `group_id` carried by any graded-answer variant. */
  function groupIdOf(answer: GradedAnswer): string {
    const inner = Object.values(answer)[0] as { group_id?: string } | undefined;
    return inner?.group_id ?? "";
  }

  function studentLabel(sheet: GradedJobSheet, index: number): string {
    const text = sheet.parsed.student_id_text?.trim();
    return text ? text : `${mn.results.detail.noStudent} ${index + 1}`;
  }

  function outcomeWord(key: string): string {
    const e = mn.results.export;
    switch (key) {
      case "correct":
        return mn.review.legend.correct;
      case "wrong":
        return e.outcomeWrong;
      case "blank":
        return e.outcomeBlank;
      case "multiple":
        return e.outcomeMultiple;
      case "partial":
        return e.outcomePartial;
      case "uncertain":
        return e.outcomeUncertain;
      default:
        return key;
    }
  }

  const counts = $derived.by(() => {
    const c = { correct: 0, wrong: 0, blank: 0 };
    if (!selected) return c;
    for (const answer of selected.graded.answers) {
      const key = outcomeKey(answer);
      if (key === "correct") c.correct += 1;
      else if (key === "blank") c.blank += 1;
      else c.wrong += 1; // wrong / multiple / partial / uncertain
    }
    return c;
  });

  function formatSummary(): string {
    return mn.results.detail.summary
      .replace("{total}", String(sheets.length))
      .replace("{avg}", avgScore.toFixed(1));
  }

  function formatCounts(): string {
    return mn.results.detail.counts
      .replace("{correct}", String(counts.correct))
      .replace("{wrong}", String(counts.wrong))
      .replace("{blank}", String(counts.blank));
  }

  async function doExport(): Promise<void> {
    if (!job || !template || !answerKey) return;
    exporting = true;
    try {
      await exportJobToXlsx({
        title: template.title,
        template,
        answerKey,
        sheets,
      });
    } catch {
      // exportJobToXlsx already surfaced the error via toast.
    } finally {
      exporting = false;
    }
  }
</script>

{#if loadFailed}
  <section class="p-8">
    <Button variant="ghost" size="sm" onclick={() => goto("/results")}>
      <ArrowLeftIcon />
      {mn.results.detail.back}
    </Button>
    <Card.Root class="border-destructive/40 mt-4 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-destructive text-base">{loadFailed}</Card.Title>
      </Card.Header>
    </Card.Root>
  </section>
{:else if job && template}
  <section class="flex h-full flex-col gap-4 p-6">
    <header class="flex items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <Button variant="ghost" size="icon" onclick={() => goto("/results")}>
          <ArrowLeftIcon />
        </Button>
        <div>
          <h2 class="text-xl font-bold">{template.title}</h2>
          <p class="text-muted-foreground text-xs">{formatSummary()}</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if job.needs_review_count > 0}
          <Button
            variant="outline"
            size="sm"
            onclick={() => goto(`/review/${job!.id}`)}
          >
            <PencilIcon />
            {mn.results.detail.reviewCta}
          </Button>
        {/if}
        <Button size="sm" disabled={exporting} onclick={doExport}>
          <FileSpreadsheetIcon />
          {exporting ? mn.results.detail.exporting : mn.results.detail.exportXlsx}
        </Button>
      </div>
    </header>

    <div class="grid flex-1 grid-cols-[280px_1fr] gap-4 overflow-hidden">
      <aside class="flex flex-col gap-1 overflow-y-auto pr-1">
        {#each sheets as sheet, i (sheet.page_index)}
          <button
            type="button"
            class="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-left text-sm transition
              {i === selectedIndex
              ? 'border-primary bg-primary/5'
              : 'hover:bg-muted/50'}"
            onclick={() => (selectedIndex = i)}
          >
            <span class="truncate">{studentLabel(sheet, i)}</span>
            <span class="flex items-center gap-2">
              {#if sheet.graded.needs_review}
                <span class="bg-destructive size-2 rounded-full"></span>
              {/if}
              <span class="tabular-nums">{sheet.graded.total_score.toFixed(1)}</span>
            </span>
          </button>
        {/each}
      </aside>

      {#if selected}
        <div class="grid grid-cols-[1fr_240px] gap-4 overflow-hidden">
          <div class="bg-muted/30 flex items-center justify-center overflow-hidden rounded-md border">
            <img
              src={assetUrl(selected.page_image_path)}
              alt={studentLabel(selected, selectedIndex)}
              class="max-h-full max-w-full object-contain"
            />
          </div>

          <div class="flex flex-col gap-3 overflow-y-auto">
            <Card.Root>
              <Card.Header class="pb-2">
                <Card.Title class="text-sm">
                  {studentLabel(selected, selectedIndex)}
                </Card.Title>
              </Card.Header>
              <Card.Content class="text-sm">
                <p class="mb-1">
                  {mn.results.detail.score}:
                  <span class="font-semibold">
                    {selected.graded.total_score.toFixed(1)}
                  </span>
                </p>
                <p class="text-muted-foreground mb-2 text-xs">{formatCounts()}</p>
                {#if selected.graded.needs_review}
                  <Badge variant="destructive">{mn.results.badge.needsReview}</Badge>
                {:else}
                  <Badge variant="secondary">{mn.results.badge.ready}</Badge>
                {/if}
              </Card.Content>
            </Card.Root>

            <div class="flex flex-col gap-1 text-sm">
              <p class="text-muted-foreground text-xs font-medium">
                {mn.results.detail.breakdown}
              </p>
              {#each selected.graded.answers as answer (groupIdOf(answer))}
                {@const key = outcomeKey(answer)}
                {@const groupId = groupIdOf(answer)}
                <div class="flex items-center justify-between gap-2 border-b py-1 last:border-0">
                  <span class="truncate">{labelByGroup.get(groupId) ?? groupId}</span>
                  <span
                    class="text-xs {key === 'correct'
                      ? 'text-emerald-600'
                      : key === 'blank'
                        ? 'text-muted-foreground'
                        : 'text-destructive'}"
                  >
                    {outcomeWord(key)}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </section>
{/if}
