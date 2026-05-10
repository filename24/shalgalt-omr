<script lang="ts">
  import { onMount } from "svelte";
  import { goto, beforeNavigate } from "$app/navigation";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import { toast } from "svelte-sonner";

  import { progress } from "$lib/stores/progress.svelte";
  import { session } from "$lib/stores/session.svelte";
  import { gradePdf, TASK_RESULT_EVENT } from "$lib/ipc/scan";
  import { pickPdf } from "$lib/picker";
  import { listTemplates } from "$lib/db/templates";
  import { createJob, completeJob, updateJobProgress } from "$lib/db/jobs";
  import { answerKeySchema } from "$lib/schemas/answerKey";
  import { mn } from "$lib/i18n";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Progress } from "$lib/components/ui/progress";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import PlayIcon from "@lucide/svelte/icons/play";
  import FileJsonIcon from "@lucide/svelte/icons/file-json";

  import type { TemplateSummary } from "$lib/types/template";
  import type { TaskResult } from "$lib/types/generated/TaskResult";
  import type { TaskStage } from "$lib/types/generated/TaskStage";
  import type { GradedJobSheet } from "$lib/types/job";

  let pdfPath = $state(session.lastPdfPath ?? "");
  let templateId = $state<number | null>(session.lastOpenedTemplateId);
  let templates = $state<TemplateSummary[]>([]);
  let answerKeyMode = $state<"paste" | "file">("paste");
  let answerKeyJson = $state("");
  let answerKeyPickedPath = $state<string | null>(null);
  let busy = $state(false);
  let activeTaskId = $state<string | null>(null);
  let activeJobId = $state<number | null>(null);
  let collected = $state<TaskResult[]>([]);
  let terminalState = $state<"running" | "done" | "failed">("running");
  let failureMessage = $state<string | null>(null);
  let needsReviewCount = $state(0);

  const last = $derived(progress.last);
  const matchesActiveTask = $derived(
    last && activeTaskId !== null && last.task_id === activeTaskId,
  );
  const stageLabel = $derived(
    last && matchesActiveTask
      ? mn.status.stage[last.stage as TaskStage]
      : null,
  );
  const pct = $derived(
    last && matchesActiveTask && last.total > 0
      ? Math.round((last.processed / last.total) * 100)
      : 0,
  );
  const selectedTemplate = $derived(
    templates.find((t) => t.id === templateId) ?? null,
  );

  let unlistenResult: UnlistenFn | null = null;

  onMount(() => {
    void loadTemplates();
    void registerResultListener();
    return () => {
      unlistenResult?.();
      unlistenResult = null;
    };
  });

  async function loadTemplates(): Promise<void> {
    try {
      templates = await listTemplates();
      // Default to the most recently opened template, or the first one.
      if (templateId === null && templates.length > 0) {
        templateId = templates[0]!.id;
      }
    } catch (e) {
      toast.error(mn.errors.unknown, { description: String(e) });
    }
  }

  async function registerResultListener(): Promise<void> {
    if (unlistenResult) return;
    unlistenResult = await listen<TaskResult>(TASK_RESULT_EVENT, (event) => {
      if (event.payload.task_id !== activeTaskId) return;
      collected = [...collected, event.payload];
    });
  }

  async function browse(): Promise<void> {
    try {
      const p = await pickPdf();
      if (p) pdfPath = p;
    } catch (e) {
      toast.error(mn.errors.unknown, { description: String(e) });
    }
  }

  async function pickAnswerKeyFile(): Promise<void> {
    try {
      const picked = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (typeof picked !== "string") return;
      answerKeyPickedPath = picked;
      const content = await readTextFile(picked);
      answerKeyJson = content;
    } catch (e) {
      toast.error(mn.grade.answerKey.readFailed, { description: String(e) });
    }
  }

  function validateAnswerKey(): { ok: boolean; message?: string } {
    if (answerKeyJson.trim() === "") {
      return { ok: false, message: mn.grade.errors.answerKeyRequired };
    }
    try {
      const parsed = JSON.parse(answerKeyJson) as unknown;
      answerKeySchema.parse(parsed);
      return { ok: true };
    } catch (e) {
      return {
        ok: false,
        message:
          e instanceof Error ? e.message : mn.grade.errors.jsonInvalid,
      };
    }
  }

  async function start(): Promise<void> {
    if (busy) {
      toast.error(mn.grade.errors.jobAlreadyRunning);
      return;
    }
    if (!pdfPath) {
      toast.error(mn.grade.errors.pdfRequired);
      return;
    }
    if (templateId === null || !selectedTemplate) {
      toast.error(mn.grade.errors.templateRequired);
      return;
    }
    const validation = validateAnswerKey();
    if (!validation.ok) {
      toast.error(mn.grade.answerKey.validateFailed, {
        description: validation.message,
      });
      return;
    }

    busy = true;
    terminalState = "running";
    failureMessage = null;
    collected = [];
    needsReviewCount = 0;

    try {
      const taskId = crypto.randomUUID();
      const job = await createJob({
        task_id: taskId,
        pdf_path: pdfPath,
        template_id: templateId,
        answer_key_json: answerKeyJson,
      });
      activeTaskId = taskId;
      activeJobId = job.id;
      session.setActiveJob(taskId);
      session.recordPdfPath(pdfPath);
      session.recordTemplate(templateId);

      await gradePdf({
        taskId,
        pdfPath,
        templateJson: JSON.stringify(selectedTemplate.schema),
        answerKeyJson,
      });
    } catch (e) {
      busy = false;
      terminalState = "failed";
      failureMessage = String(e);
      toast.error(mn.grade.errors.startFailed, { description: String(e) });
    }
  }

  // Watch progress events and persist to the jobs table; on terminal stage,
  // write the collected sheets and unlock the form.
  $effect(() => {
    const p = progress.last;
    if (!p || activeTaskId === null || p.task_id !== activeTaskId) return;
    if (p.stage === "done") {
      void finalizeJob("done", null);
    } else if (p.stage === "failed") {
      void finalizeJob("failed", p.message ?? "unknown error");
    } else if (busy) {
      // Live progress — best-effort DB update; ignore errors so the UI keeps
      // animating even if SQLite is briefly contended.
      void updateJobProgress(activeTaskId, {
        processed_pages: p.processed,
        total_pages: p.total > 0 ? p.total : null,
      }).catch(() => {});
    }
  });

  async function finalizeJob(
    status: "done" | "failed",
    error: string | null,
  ): Promise<void> {
    if (!activeTaskId) return;
    const taskId = activeTaskId;
    const sheets: GradedJobSheet[] = collected.map((r) => ({
      page_index: r.page_index,
      page_image_path: r.page_image_path,
      parsed: r.parsed,
      graded: r.graded,
      reviewed: false,
    }));
    needsReviewCount = sheets.filter((s) => s.graded.needs_review).length;

    try {
      await completeJob(taskId, {
        status,
        graded_sheets: status === "done" ? sheets : null,
        needs_review_count: needsReviewCount,
        error_message: error,
      });
    } catch (e) {
      toast.error(mn.errors.unknown, { description: String(e) });
    }

    busy = false;
    terminalState = status;
    failureMessage = error;
  }

  function gotoReview(): void {
    if (activeJobId === null) return;
    void goto(`/review/${activeJobId}`);
  }

  function gotoResults(): void {
    void goto("/results");
  }

  beforeNavigate((nav) => {
    if (busy && nav.to?.url.pathname !== "/grade") {
      const ok = window.confirm(mn.grade.errors.jobAlreadyRunning);
      if (!ok) nav.cancel();
    }
  });

  function formatSummary(total: number, flagged: number): string {
    return mn.grade.completed.summary
      .replace("{total}", String(total))
      .replace("{flagged}", String(flagged));
  }

  function formatReviewCta(flagged: number): string {
    return mn.grade.completed.reviewCta.replace("{flagged}", String(flagged));
  }

  function formatFailure(message: string): string {
    return mn.grade.failed.detail.replace("{message}", message);
  }
</script>

<section class="p-8">
  <header class="mb-6">
    <h2 class="text-2xl font-bold">{mn.grade.title}</h2>
    <p class="text-muted-foreground text-sm">{mn.grade.subtitle}</p>
  </header>

  <Card.Root class="max-w-2xl">
    <Card.Content class="space-y-5 pt-6">
      <div class="space-y-1.5">
        <Label for="pdf-path">{mn.grade.pdfLabel}</Label>
        <div class="flex gap-2">
          <Input
            id="pdf-path"
            bind:value={pdfPath}
            placeholder={mn.grade.pdfPlaceholder}
            class="flex-1"
            readonly
          />
          <Button type="button" variant="outline" onclick={browse}>
            <FolderOpenIcon />
            {mn.grade.pdfBrowse}
          </Button>
        </div>
      </div>

      <div class="space-y-1.5">
        <Label for="template-id">{mn.grade.templateLabel}</Label>
        {#if templates.length === 0}
          <p class="text-muted-foreground text-sm">{mn.grade.templateEmpty}</p>
        {:else}
          <select
            id="template-id"
            bind:value={templateId}
            class="border-input bg-background text-foreground focus-visible:ring-ring h-9 w-full rounded-md border px-3 py-1 text-sm shadow-sm focus-visible:ring-1 focus-visible:outline-none"
          >
            <option value={null} disabled>{mn.grade.templatePlaceholder}</option>
            {#each templates as t (t.id)}
              <option value={t.id}>{t.title}</option>
            {/each}
          </select>
        {/if}
      </div>

      <div class="space-y-1.5">
        <Label>{mn.grade.answerKey.label}</Label>
        <p class="text-muted-foreground text-xs">{mn.grade.answerKey.help}</p>
        <Tabs.Root bind:value={answerKeyMode} class="mt-2">
          <Tabs.List>
            <Tabs.Trigger value="paste">{mn.grade.answerKey.tabPaste}</Tabs.Trigger>
            <Tabs.Trigger value="file">{mn.grade.answerKey.tabFile}</Tabs.Trigger>
          </Tabs.List>
          <Tabs.Content value="paste" class="mt-2">
            <textarea
              bind:value={answerKeyJson}
              placeholder={mn.grade.answerKey.paste}
              rows="6"
              class="border-input bg-background text-foreground focus-visible:ring-ring min-h-32 w-full rounded-md border px-3 py-2 font-mono text-xs shadow-sm focus-visible:ring-1 focus-visible:outline-none"
            ></textarea>
          </Tabs.Content>
          <Tabs.Content value="file" class="mt-2 space-y-2">
            <Button type="button" variant="outline" onclick={pickAnswerKeyFile}>
              <FileJsonIcon />
              {mn.grade.answerKey.pickFile}
            </Button>
            {#if answerKeyPickedPath}
              <p class="text-muted-foreground text-xs">
                {mn.grade.answerKey.pickedFile}: {answerKeyPickedPath}
              </p>
            {/if}
          </Tabs.Content>
        </Tabs.Root>
      </div>
    </Card.Content>
    <Card.Footer class="flex items-center justify-between">
      <Button onclick={start} disabled={busy || !pdfPath || templateId === null}>
        <PlayIcon />
        {busy ? mn.grade.starting : mn.grade.start}
      </Button>
      {#if activeTaskId}
        <Badge variant="secondary">task_id {activeTaskId.slice(0, 8)}</Badge>
      {/if}
    </Card.Footer>
  </Card.Root>

  {#if matchesActiveTask && last}
    <Card.Root class="mt-6 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-base">{mn.grade.progress.title}</Card.Title>
      </Card.Header>
      <Card.Content class="space-y-3">
        <Progress value={pct} />
        <div class="text-muted-foreground flex justify-between text-sm">
          <span>{mn.grade.progress.stage}: {stageLabel ?? last.stage}</span>
          <span>{last.processed} / {last.total} {mn.grade.progress.pageCounter}</span>
        </div>
        {#if last.message}
          <p class="text-muted-foreground text-xs">{last.message}</p>
        {/if}
      </Card.Content>
    </Card.Root>
  {/if}

  {#if terminalState === "done" && activeJobId !== null}
    <Card.Root class="mt-6 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-base">{mn.grade.completed.title}</Card.Title>
        <Card.Description>
          {formatSummary(collected.length, needsReviewCount)}
        </Card.Description>
      </Card.Header>
      <Card.Footer class="gap-2">
        {#if needsReviewCount > 0}
          <Button onclick={gotoReview}>
            {formatReviewCta(needsReviewCount)}
          </Button>
        {/if}
        <Button variant="outline" onclick={gotoResults}>
          {mn.grade.completed.seeResults}
        </Button>
      </Card.Footer>
    </Card.Root>
  {:else if terminalState === "failed" && failureMessage}
    <Card.Root class="border-destructive/40 mt-6 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-destructive text-base">
          {mn.grade.failed.title}
        </Card.Title>
        <Card.Description>{formatFailure(failureMessage)}</Card.Description>
      </Card.Header>
    </Card.Root>
  {/if}
</section>
