<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto, beforeNavigate } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import { mn } from "$lib/i18n";
  import {
    getJobById,
    parseGradedSheets,
    updateGradedSheet,
  } from "$lib/db/jobs";
  import { getTemplate } from "$lib/db/templates";
  import { regradeSheet } from "$lib/ipc/scan";
  import { parseStoredAnswerKeys } from "$lib/results/storedAnswerKeys";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";

  import ReviewCanvas from "$lib/components/review/ReviewCanvas.svelte";
  import PageThumbnail from "$lib/components/review/PageThumbnail.svelte";
  import type { Job, GradedJobSheet } from "$lib/types/job";
  import type { OmrTemplate } from "$lib/types/template";
  import type { AnswerKey } from "$lib/types/generated/AnswerKey";
  import type { BubbleReading } from "$lib/types/generated/BubbleReading";
  import SaveIcon from "@lucide/svelte/icons/save";

  let job = $state<Job | null>(null);
  let sheets = $state<GradedJobSheet[]>([]);
  let template = $state<OmrTemplate | null>(null);
  // Every variant's key (the job may grade a mixed-variant batch); the key for
  // the open sheet is resolved by its decoded variant.
  let answerKeys = $state<AnswerKey[]>([]);
  let selectedIndex = $state<number>(0);
  let dirtyIndexes = $state<Set<number>>(new Set());
  let canvasWidth = $state(0);
  let canvasHeight = $state(0);
  let canvasContainer: HTMLDivElement | null = $state(null);
  let loadFailed = $state<string | null>(null);

  const jobIdParam = $derived(Number(page.params.job_id));
  const selected = $derived(sheets[selectedIndex] ?? null);
  // The key for the open sheet, matched by its decoded variant. `null` when the
  // variant is unresolved (no matching key) — the canvas then drops the
  // correct-answer overlay and re-grading is disabled for that sheet.
  const selectedAnswerKey = $derived(
    answerKeys.find((k) => k.variant === selected?.parsed.variant) ??
      (answerKeys.length === 1 ? answerKeys[0]! : null),
  );
  const reviewedCount = $derived(sheets.filter((s) => s.reviewed).length);
  const flaggedCount = $derived(
    sheets.filter((s) => s.graded.needs_review && !s.reviewed).length,
  );

  onMount(() => {
    void loadJob();
  });

  // Track canvas size so the Konva stage gets pixel-accurate width/height.
  //
  // The container lives inside the `{:else if job && …}` block, so it does not
  // exist yet when the component first mounts — the job loads asynchronously.
  // Observing it once in `onMount` would bind to a `null` element and never
  // fire, leaving the stage at 0×0 and the canvas stuck on its "…" placeholder.
  // Run reactively instead: the effect re-runs when `bind:this` populates
  // `canvasContainer`, seeds the initial size, and observes for later resizes.
  //
  // The container chain uses `min-h-0`/`min-w-0` so `flex-1`+`overflow-hidden`
  // actually bound the box independently of the Konva stage inside it. Without
  // that, the stage would push the container larger, the observer would read
  // the larger size, grow the stage again, and the page would expand forever.
  // We still guard here: only write on an actual change, and coalesce in a
  // rAF, which also silences the "ResizeObserver loop" console warning.
  $effect(() => {
    const el = canvasContainer;
    if (!el) return;
    let frame = 0;
    const measure = () => {
      frame = 0;
      const w = el.clientWidth;
      const h = el.clientHeight;
      if (w !== canvasWidth) canvasWidth = w;
      if (h !== canvasHeight) canvasHeight = h;
    };
    measure();
    const ro = new ResizeObserver(() => {
      if (frame === 0) frame = requestAnimationFrame(measure);
    });
    ro.observe(el);
    return () => {
      if (frame !== 0) cancelAnimationFrame(frame);
      ro.disconnect();
    };
  });

  async function loadJob(): Promise<void> {
    try {
      const j = await getJobById(jobIdParam);
      if (!j) {
        loadFailed = mn.review.notFound;
        return;
      }
      if (j.status !== "done") {
        loadFailed = mn.review.notFinishedYet;
        return;
      }
      // Resolve the template and answer keys before flipping the render gate.
      // If either is missing the gate (`job && template && answerKeys.length`)
      // can never become true, so surface an explicit error instead of leaving
      // the page blank — which reads as "stuck loading".
      const tpl = await getTemplate(j.template_id);
      if (!tpl) {
        loadFailed = mn.review.templateMissing;
        return;
      }
      const keys = parseStoredAnswerKeys(j.answer_key_json);
      if (keys.length === 0) {
        loadFailed = mn.review.answerKeyMissing;
        return;
      }

      sheets = parseGradedSheets(j);
      // Open the first flagged sheet by default; otherwise the first sheet.
      const flaggedIdx = sheets.findIndex((s) => s.graded.needs_review);
      selectedIndex = flaggedIdx >= 0 ? flaggedIdx : 0;
      template = tpl.schema;
      answerKeys = keys;
      job = j;
    } catch (e) {
      loadFailed = String(e);
    }
  }

  async function applyOverride(next: BubbleReading[]): Promise<void> {
    if (!template || !selected || !job) return;
    // Re-grade against the sheet's own variant key. With no resolved key there
    // is nothing to score against, so surface that instead of mis-grading.
    if (!selectedAnswerKey) {
      toast.error(mn.review.regradeNoVariant);
      return;
    }
    const idx = selectedIndex;
    const updatedParsed = { ...selected.parsed, readings: next };

    try {
      const newGraded = await regradeSheet({
        templateJson: JSON.stringify(template),
        parsedSheetJson: JSON.stringify(updatedParsed),
        answerKeyJson: JSON.stringify(selectedAnswerKey),
      });
      sheets = sheets.map((s, i) =>
        i === idx ? { ...s, parsed: updatedParsed, graded: newGraded } : s,
      );
      dirtyIndexes = new Set([...dirtyIndexes, idx]);
    } catch (e) {
      toast.error(mn.review.regradeFailed, { description: String(e) });
    }
  }

  /**
   * Assign a variant to the open sheet and re-grade against that variant's key.
   * Lets the teacher resolve a sheet whose variant the CV could not read (or
   * read wrong) instead of re-scanning it.
   */
  async function assignVariant(nextVariant: string): Promise<void> {
    if (!template || !selected || !job) return;
    const key = answerKeys.find((k) => k.variant === nextVariant);
    if (!key) return;
    const idx = selectedIndex;
    const updatedParsed = { ...selected.parsed, variant: nextVariant };

    try {
      const newGraded = await regradeSheet({
        templateJson: JSON.stringify(template),
        parsedSheetJson: JSON.stringify(updatedParsed),
        answerKeyJson: JSON.stringify(key),
      });
      sheets = sheets.map((s, i) =>
        i === idx ? { ...s, parsed: updatedParsed, graded: newGraded } : s,
      );
      dirtyIndexes = new Set([...dirtyIndexes, idx]);
    } catch (e) {
      toast.error(mn.review.regradeFailed, { description: String(e) });
    }
  }

  async function saveCurrent(): Promise<void> {
    if (!job || !selected) return;
    const idx = selectedIndex;
    const sheet: GradedJobSheet = { ...selected, reviewed: true };
    try {
      await updateGradedSheet(job.id, sheet.page_index, sheet);
      sheets = sheets.map((s, i) => (i === idx ? sheet : s));
      const dirty = new Set(dirtyIndexes);
      dirty.delete(idx);
      dirtyIndexes = dirty;
      toast.success(mn.review.saveSuccess);
    } catch (e) {
      toast.error(mn.review.saveFailed, { description: String(e) });
    }
  }

  async function saveAllAndExit(): Promise<void> {
    if (!job) return;
    try {
      // Persist whichever sheets carry unsaved overrides.
      for (const idx of dirtyIndexes) {
        const sheet = sheets[idx];
        if (!sheet) continue;
        await updateGradedSheet(job.id, sheet.page_index, {
          ...sheet,
          reviewed: true,
        });
      }
      dirtyIndexes = new Set();
      toast.success(mn.review.saveSuccess);
      void goto("/results");
    } catch (e) {
      toast.error(mn.review.saveFailed, { description: String(e) });
    }
  }

  function selectIndex(i: number): void {
    selectedIndex = i;
  }

  function formatCounter(reviewed: number, total: number): string {
    return mn.review.counter
      .replace("{reviewed}", String(reviewed))
      .replace("{total}", String(total));
  }

  function formatFlagged(count: number): string {
    return mn.review.flaggedSummary.replace("{count}", String(count));
  }

  beforeNavigate((nav) => {
    if (dirtyIndexes.size > 0) {
      const ok = window.confirm(mn.dialog.unsavedChanges.body);
      if (!ok) nav.cancel();
    }
  });
</script>

{#if loadFailed}
  <section class="p-8">
    <Card.Root class="border-destructive/40 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-destructive text-base">
          {loadFailed}
        </Card.Title>
      </Card.Header>
    </Card.Root>
  </section>
{:else if !job}
  <section class="p-8">
    <p class="text-muted-foreground text-sm">{mn.review.loading}</p>
  </section>
{:else if template && answerKeys.length > 0}
  <section class="flex h-full flex-col p-4">
    <header class="mb-4 flex items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold">{mn.review.title}</h2>
        <p class="text-muted-foreground text-xs">{mn.review.subtitle}</p>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="secondary">{formatCounter(reviewedCount, sheets.length)}</Badge>
        {#if flaggedCount > 0}
          <Badge variant="destructive">{formatFlagged(flaggedCount)}</Badge>
        {/if}
        <Button onclick={saveAllAndExit}>
          <SaveIcon />
          {mn.review.saveAndExit}
        </Button>
      </div>
    </header>

    <div class="grid min-h-0 flex-1 grid-cols-[280px_1fr] gap-4 overflow-hidden">
      <aside class="flex min-h-0 flex-col gap-2 overflow-y-auto pr-1">
        {#each sheets as sheet, i (sheet.page_index)}
          <PageThumbnail
            {sheet}
            selected={i === selectedIndex}
            onSelect={() => selectIndex(i)}
          />
        {/each}
        {#if sheets.length === 0}
          <p class="text-muted-foreground text-sm">{mn.review.emptyAllReviewed}</p>
        {/if}
      </aside>

      <div class="flex min-h-0 min-w-0 flex-col gap-2 overflow-hidden">
        <div class="flex items-center justify-between text-xs">
          <p class="text-muted-foreground">{mn.review.bubbleHint}</p>
          <div class="flex gap-2">
            <span class="flex items-center gap-1">
              <span class="bg-emerald-500/70 inline-block size-3 rounded-full"></span>
              {mn.review.legend.correct}
            </span>
            <span class="flex items-center gap-1">
              <span class="bg-red-500/70 inline-block size-3 rounded-full"></span>
              {mn.review.legend.wrong}
            </span>
            <span class="flex items-center gap-1">
              <span class="bg-amber-500/70 inline-block size-3 rounded-full"></span>
              {mn.review.legend.uncertain}
            </span>
          </div>
        </div>
        <div bind:this={canvasContainer} class="min-h-0 min-w-0 flex-1 overflow-hidden">
          {#if selected}
            <ReviewCanvas
              width={canvasWidth}
              height={canvasHeight}
              {template}
              pageImagePath={selected.page_image_path}
              readings={selected.parsed.readings}
              graded={selected.graded}
              answerKey={selectedAnswerKey}
              onOverride={applyOverride}
            />
          {/if}
        </div>
        {#if selected}
          <footer class="flex items-center justify-between gap-3 border-t pt-2 text-sm">
            <p>
              {mn.review.pageHeader} {selected.page_index + 1} —
              {mn.review.score}: {selected.graded.total_score.toFixed(1)}
            </p>
            <div class="flex items-center gap-2">
              {#if answerKeys.length > 1}
                <label class="text-muted-foreground flex items-center gap-1.5 text-xs">
                  {mn.review.variantLabel}
                  <select
                    class="border-input bg-background text-foreground h-8 rounded-md border px-2 text-sm"
                    value={selected.parsed.variant ?? ""}
                    onchange={(e) =>
                      assignVariant((e.target as HTMLSelectElement).value)}
                  >
                    {#if !selectedAnswerKey}
                      <option value="" disabled>
                        {mn.review.variantUnresolved}
                      </option>
                    {/if}
                    {#each answerKeys as k (k.variant)}
                      <option value={k.variant}>{k.variant}</option>
                    {/each}
                  </select>
                </label>
              {/if}
              <Button size="sm" onclick={saveCurrent}>
                <SaveIcon />
                {mn.review.saveSheet}
              </Button>
            </div>
          </footer>
        {/if}
      </div>
    </div>
  </section>
{/if}
