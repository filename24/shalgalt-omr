<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import { mn } from "$lib/i18n";
  import { listExams, createExam, deleteExam } from "$lib/db/exams";
  import { listTemplates } from "$lib/db/templates";
  import type { ExamSummary } from "$lib/types/exam";
  import type { TemplateSummary } from "$lib/types/template";
  import { examNameSchema } from "$lib/types/exam";
  import { pickShalgalt, pickShalgaltSavePath } from "$lib/picker";
  import { assembleProject } from "$lib/project";
  import { projectExport } from "$lib/ipc/project";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";

  let exams = $state<ExamSummary[] | null>(null);
  let templates = $state<TemplateSummary[]>([]);

  // Create-dialog state.
  let createOpen = $state(false);
  let newName = $state("");
  let newTemplateId = $state<number | null>(null);
  let creating = $state(false);

  // Delete-dialog state.
  let deleteTarget = $state<ExamSummary | null>(null);
  let deleting = $state(false);

  // Id of the exam currently being exported, so its row button can show busy.
  let exportingId = $state<number | null>(null);

  // Export-dialog state. The dialog is open while `exportTarget` is set; it lets
  // the user choose whether to password-protect (age-encrypt) the `.shalgalt`.
  let exportTarget = $state<ExamSummary | null>(null);
  let exportEncrypt = $state(false);
  let exportPassphrase = $state("");
  let exportPassphraseConfirm = $state("");

  const exportPassphraseMismatch = $derived(
    exportEncrypt &&
      exportPassphraseConfirm.length > 0 &&
      exportPassphrase !== exportPassphraseConfirm,
  );

  // Block confirm while busy, or — when encryption is on — until a non-empty
  // passphrase is entered and confirmed identically (a typo'd passphrase would
  // make the file permanently unreadable).
  const canExport = $derived(
    exportingId === null &&
      (!exportEncrypt ||
        (exportPassphrase.length > 0 &&
          exportPassphrase === exportPassphraseConfirm)),
  );

  const canCreate = $derived(
    !creating &&
      templates.length > 0 &&
      newTemplateId !== null &&
      examNameSchema.safeParse(newName.trim()).success,
  );

  onMount(() => {
    void load();
  });

  async function load(): Promise<void> {
    try {
      const [examRows, templateRows] = await Promise.all([
        listExams(),
        listTemplates(),
      ]);
      exams = examRows;
      templates = templateRows;
      if (newTemplateId === null && templateRows.length > 0) {
        newTemplateId = templateRows[0]!.id;
      }
    } catch (e) {
      toast.error(mn.errors.unknown, { description: String(e) });
      exams = [];
    }
  }

  function openCreate(): void {
    newName = "";
    newTemplateId = templates.length > 0 ? templates[0]!.id : null;
    createOpen = true;
  }

  async function confirmCreate(): Promise<void> {
    if (!canCreate || newTemplateId === null) return;
    creating = true;
    try {
      const exam = await createExam({
        name: newName.trim(),
        template_id: newTemplateId,
      });
      toast.success(mn.exams.toasts.created);
      createOpen = false;
      void goto(`/exams/${exam.id}`);
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      creating = false;
    }
  }

  function openDelete(exam: ExamSummary): void {
    deleteTarget = exam;
  }

  async function confirmDelete(): Promise<void> {
    if (!deleteTarget) return;
    deleting = true;
    try {
      await deleteExam(deleteTarget.id);
      toast.success(mn.exams.toasts.deleted);
      deleteTarget = null;
      await load();
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      deleting = false;
    }
  }

  /**
   * Pick a `.shalgalt` file and hand it to the existing "Open with" import
   * page, which owns the passphrase prompt + `restoreProject` flow. Routing
   * there instead of re-opening inline keeps a single import code path.
   */
  async function openFile(): Promise<void> {
    const path = await pickShalgalt();
    if (!path) return;
    void goto(`/exams/import?path=${encodeURIComponent(path)}`);
  }

  /** Open the export dialog for one exam, resetting the encryption fields. */
  function openExport(exam: ExamSummary): void {
    exportTarget = exam;
    exportEncrypt = false;
    exportPassphrase = "";
    exportPassphraseConfirm = "";
  }

  function closeExport(): void {
    exportTarget = null;
  }

  /**
   * Export the dialog's exam to a `.shalgalt` container: assemble its template
   * (and any bundled job data) into a manifest + entries, pick a destination,
   * then write the archive via `project_export`. When the user enabled
   * encryption, the passphrase is passed to both `assembleProject` (so the
   * manifest's `encrypted` flag is set) and `project_export` (so the payload is
   * age-encrypted); the Rust writer enforces that the two agree.
   */
  async function confirmExport(): Promise<void> {
    const exam = exportTarget;
    if (!exam || !canExport) return;
    const passphrase = exportEncrypt ? exportPassphrase : undefined;
    exportingId = exam.id;
    try {
      const { manifestJson, entries } = await assembleProject({
        templateId: exam.template_id,
        title: exam.name,
        passphrase,
      });
      const outputPath = await pickShalgaltSavePath(`${exam.name}.shalgalt`);
      if (!outputPath) return;
      await projectExport({ manifestJson, entries, outputPath, passphrase });
      toast.success(mn.exams.toasts.exported, { description: exam.name });
      exportTarget = null;
    } catch (e) {
      toast.error(mn.exams.toasts.exportFailed, { description: String(e) });
    } finally {
      exportingId = null;
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString();
  }
</script>

<section class="container mx-auto space-y-6 p-6 lg:p-8">
  <header class="flex items-start justify-between gap-4">
    <div class="space-y-1">
      <h2 class="text-2xl font-bold tracking-tight">{mn.exams.title}</h2>
      <p class="text-muted-foreground text-sm">{mn.exams.subtitle}</p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" onclick={openFile}>
        <FolderOpenIcon />
        {mn.exams.openFile}
      </Button>
      <Button onclick={openCreate}>
        <PlusIcon />
        {mn.exams.newExam}
      </Button>
    </div>
  </header>

  <Card.Root>
    <Card.Content class="pt-6">
      {#if exams === null}
        <p class="text-muted-foreground text-sm">…</p>
      {:else if exams.length === 0}
        <p class="text-muted-foreground py-8 text-center text-sm">
          {mn.exams.empty}
        </p>
      {:else}
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-muted-foreground border-b text-left">
                <th class="px-2 py-2 font-medium">{mn.exams.column.name}</th>
                <th class="px-2 py-2 font-medium">{mn.exams.column.template}</th>
                <th class="px-2 py-2 font-medium">{mn.exams.column.variants}</th>
                <th class="px-2 py-2 font-medium">{mn.exams.column.created}</th>
                <th class="px-2 py-2"></th>
              </tr>
            </thead>
            <tbody>
              {#each exams as exam (exam.id)}
                <tr class="hover:bg-muted/50 border-b last:border-0">
                  <td class="px-2 py-2">
                    <a
                      href={`/exams/${exam.id}`}
                      class="font-medium hover:underline"
                    >
                      {exam.name}
                    </a>
                  </td>
                  <td class="text-muted-foreground px-2 py-2">
                    {exam.template_title}
                  </td>
                  <td class="px-2 py-2">
                    <Badge variant="secondary">{exam.variant_count}</Badge>
                  </td>
                  <td class="text-muted-foreground px-2 py-2">
                    {formatDate(exam.created_at)}
                  </td>
                  <td class="px-2 py-2 text-right">
                    <Button
                      variant="ghost"
                      size="icon"
                      onclick={() => openExport(exam)}
                      disabled={exportingId === exam.id}
                      aria-label={mn.exams.exportExam}
                    >
                      <DownloadIcon class="size-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onclick={() => openDelete(exam)}
                      aria-label={mn.exams.deleteExam}
                    >
                      <Trash2Icon class="text-destructive size-4" />
                    </Button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </Card.Content>
  </Card.Root>
</section>

<Dialog.Root bind:open={createOpen}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{mn.exams.create.title}</Dialog.Title>
    </Dialog.Header>
    <div class="space-y-4 py-2">
      <div class="space-y-1.5">
        <Label for="exam-name">{mn.exams.create.nameLabel}</Label>
        <Input
          id="exam-name"
          bind:value={newName}
          placeholder={mn.exams.create.namePlaceholder}
        />
      </div>
      <div class="space-y-1.5">
        <Label for="exam-template">{mn.exams.create.templateLabel}</Label>
        {#if templates.length === 0}
          <p class="text-muted-foreground text-sm">
            {mn.exams.create.templateEmpty}
          </p>
        {:else}
          <select
            id="exam-template"
            bind:value={newTemplateId}
            class="border-input bg-background text-foreground focus-visible:ring-ring h-9 w-full rounded-md border px-3 py-1 text-sm shadow-sm focus-visible:ring-1 focus-visible:outline-none"
          >
            <option value={null} disabled
              >{mn.exams.create.templatePlaceholder}</option
            >
            {#each templates as t (t.id)}
              <option value={t.id}>{t.title}</option>
            {/each}
          </select>
        {/if}
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (createOpen = false)}>
        {mn.exams.create.cancel}
      </Button>
      <Button onclick={confirmCreate} disabled={!canCreate}>
        {mn.exams.create.confirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root
  open={exportTarget !== null}
  onOpenChange={(o) => {
    if (!o) closeExport();
  }}
>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{mn.exams.export.title}</Dialog.Title>
      <Dialog.Description>{mn.exams.export.subtitle}</Dialog.Description>
    </Dialog.Header>
    <div class="space-y-4 py-2">
      <label class="flex items-center gap-2 text-sm font-medium">
        <input
          type="checkbox"
          bind:checked={exportEncrypt}
          class="border-input text-primary focus-visible:ring-ring size-4 rounded border focus-visible:ring-1 focus-visible:outline-none"
        />
        {mn.exams.export.encryptLabel}
      </label>
      {#if exportEncrypt}
        <p class="text-muted-foreground text-xs">
          {mn.exams.export.encryptHint}
        </p>
        <div class="space-y-1.5">
          <Label for="export-passphrase">
            {mn.exams.export.passphraseLabel}
          </Label>
          <Input
            id="export-passphrase"
            type="password"
            bind:value={exportPassphrase}
            placeholder={mn.exams.export.passphrasePlaceholder}
          />
        </div>
        <div class="space-y-1.5">
          <Label for="export-passphrase-confirm">
            {mn.exams.export.confirmLabel}
          </Label>
          <Input
            id="export-passphrase-confirm"
            type="password"
            bind:value={exportPassphraseConfirm}
            placeholder={mn.exams.export.confirmPlaceholder}
          />
          {#if exportPassphraseMismatch}
            <p class="text-destructive text-xs">
              {mn.exams.export.mismatch}
            </p>
          {/if}
        </div>
      {/if}
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={closeExport}>
        {mn.exams.export.cancel}
      </Button>
      <Button onclick={confirmExport} disabled={!canExport}>
        {mn.exams.export.confirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root
  open={deleteTarget !== null}
  onOpenChange={(o) => {
    if (!o) deleteTarget = null;
  }}
>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{mn.exams.deleteExamConfirm.title}</Dialog.Title>
      <Dialog.Description>{mn.exams.deleteExamConfirm.body}</Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (deleteTarget = null)}>
        {mn.exams.deleteExamConfirm.cancel}
      </Button>
      <Button variant="destructive" onclick={confirmDelete} disabled={deleting}>
        {mn.exams.deleteExamConfirm.confirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
