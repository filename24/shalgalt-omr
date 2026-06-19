<script lang="ts">
  /**
   * Scan-to-answer-key review screen (P4-06).
   *
   * The teacher picks a single-page scan of a hand-filled OMR card; the CV pipeline
   * reads it and we show a side-by-side review (scan preview + read answers). On save
   * we persist through the same `upsertAnswerKey` path the manual editor uses, so there
   * is exactly one persistence route for answer keys.
   */
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import { mn } from "$lib/i18n";
  import { getExamById } from "$lib/db/exams";
  import { getTemplate } from "$lib/db/templates";
  import { upsertAnswerKey } from "$lib/db/answerKeys";
  import { answersSchema, type Exam } from "$lib/types/exam";
  import { scanAnswerKey, type AnswerKeyImportSummary } from "$lib/ipc/scan";
  import { pickScanSource } from "$lib/picker";
  import { assetUrl } from "$lib/fs/templateAssets";
  import { progress } from "$lib/stores/progress.svelte";
  import type { OmrTemplate, BubbleGroup } from "$lib/types/template";
  import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";

  import AnswerKeyReview from "$lib/components/exams/AnswerKeyReview.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import ScanLineIcon from "@lucide/svelte/icons/scan-line";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";

  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const examId = $derived(data.examId);
  const variant = $derived(data.variant);

  let exam = $state<Exam | null>(null);
  let template = $state<OmrTemplate | null>(null);
  let loadFailed = $state<string | null>(null);

  const questionGroups = $derived<BubbleGroup[]>(
    template ? template.groups.filter((g) => g.kind === "question") : [],
  );

  let scanning = $state(false);
  let saving = $state(false);
  let activeTaskId = $state<string | null>(null);
  let summary = $state<AnswerKeyImportSummary | null>(null);
  let previewUrl = $state("");

  // Live stage label, only while our scan task is the one reporting.
  const last = $derived(progress.last);
  const stageLabel = $derived(
    scanning && last && activeTaskId !== null && last.task_id === activeTaskId
      ? mn.status.stage[last.stage]
      : null,
  );

  $effect(() => {
    void examId;
    void variant;
    void load();
  });

  async function load(): Promise<void> {
    loadFailed = null;
    summary = null;
    try {
      const e = await getExamById(examId);
      if (!e) {
        loadFailed = mn.exams.detail.notFound;
        return;
      }
      exam = e;
      const tpl = await getTemplate(e.template_id);
      template = tpl ? tpl.schema : null;
      if (!template) loadFailed = mn.exams.answerKey.templateMissing;
    } catch (err) {
      loadFailed = String(err);
    }
  }

  /** Map an `AppError` code to its Mongolian message, falling back to a generic one. */
  function scanErrorMessage(e: unknown): string {
    const code = (e as { code?: string })?.code;
    const table = mn.errors as Record<string, string>;
    return (code && table[code]) || mn.errors.unknown;
  }

  async function startScan(): Promise<void> {
    if (!template || scanning) return;
    const path = await pickScanSource();
    if (!path) return;

    const taskId = crypto.randomUUID();
    activeTaskId = taskId;
    scanning = true;
    summary = null;
    try {
      const result = await scanAnswerKey({
        taskId,
        pdfPath: path,
        templateJson: JSON.stringify(template),
        examId,
        variant,
      });
      summary = result;
      previewUrl = assetUrl(result.preview_path);
    } catch (e) {
      toast.error(scanErrorMessage(e), { description: String(e) });
    } finally {
      scanning = false;
    }
  }

  async function handleSave(entries: AnswerKeyEntry[]): Promise<void> {
    if (!exam) return;
    let validated: AnswerKeyEntry[];
    try {
      validated = answersSchema.parse(entries);
    } catch (e) {
      toast.error(mn.exams.detail.validateFailed, {
        description: e instanceof Error ? e.message : undefined,
      });
      return;
    }

    saving = true;
    try {
      await upsertAnswerKey({ exam_id: exam.id, variant, answers: validated });
      toast.success(mn.exams.answerKey.scan.saved);
      void goto(`/exams/${exam.id}`);
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      saving = false;
    }
  }

  function cancel(): void {
    void goto(`/exams/${examId}`);
  }
</script>

<section class="container mx-auto space-y-6 p-6 lg:p-8">
  <header class="flex flex-wrap items-center justify-between gap-3">
    <div class="space-y-1">
      <div class="flex items-center gap-2">
        <h2 class="text-2xl font-bold tracking-tight">
          {mn.exams.answerKey.scan.title}
        </h2>
        <Badge variant="secondary">{variant}</Badge>
      </div>
      <p class="text-muted-foreground text-sm">
        {mn.exams.answerKey.scan.subtitle}
      </p>
    </div>
    <Button variant="ghost" size="sm" onclick={cancel}>
      <ArrowLeftIcon />
      {mn.exams.detail.title}
    </Button>
  </header>

  {#if loadFailed}
    <Card.Root class="border-destructive/40 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-destructive text-base">{loadFailed}</Card.Title>
      </Card.Header>
    </Card.Root>
  {:else if summary}
    <AnswerKeyReview
      {questionGroups}
      {summary}
      {previewUrl}
      {saving}
      onSave={handleSave}
      onCancel={cancel}
    />
  {:else}
    <Card.Root class="max-w-xl">
      <Card.Content class="flex flex-col items-center gap-4 py-10 text-center">
        <ScanLineIcon class="text-muted-foreground size-10" />
        <p class="text-muted-foreground text-sm">
          {mn.exams.answerKey.scan.subtitle}
        </p>
        <Button onclick={startScan} disabled={scanning || template === null}>
          <ScanLineIcon />
          {scanning
            ? (stageLabel ?? mn.exams.answerKey.scan.scanning)
            : mn.exams.answerKey.scan.pickPdf}
        </Button>
      </Card.Content>
    </Card.Root>
  {/if}
</section>
