<script lang="ts">
  import { toast } from "svelte-sonner";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";

  import { mn } from "$lib/i18n";
  import { getExamById, updateExam } from "$lib/db/exams";
  import {
    listAnswerKeysByExam,
    upsertAnswerKey,
    deleteAnswerKey,
  } from "$lib/db/answerKeys";
  import { getTemplate } from "$lib/db/templates";
  import {
    examNameSchema,
    answersSchema,
    type Exam,
    type AnswerKeyRecord,
  } from "$lib/types/exam";
  import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Tabs from "$lib/components/ui/tabs";
  import * as Dialog from "$lib/components/ui/dialog";
  import SaveIcon from "@lucide/svelte/icons/save";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import FileJsonIcon from "@lucide/svelte/icons/file-json";

  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const examId = $derived(data.examId);

  let exam = $state<Exam | null>(null);
  let templateTitle = $state("");
  let keys = $state<AnswerKeyRecord[]>([]);
  let loadFailed = $state<string | null>(null);

  // Rename state.
  let nameDraft = $state("");
  let renaming = $state(false);

  // Add/edit-variant state. `editingId` null means "adding new".
  let formOpen = $state(false);
  let editingId = $state<number | null>(null);
  let variantDraft = $state("");
  let answerMode = $state<"paste" | "file">("paste");
  let answerJson = $state("");
  let pickedPath = $state<string | null>(null);
  let savingVariant = $state(false);

  // Delete-variant state.
  let deleteTarget = $state<AnswerKeyRecord | null>(null);
  let deletingVariant = $state(false);

  const canSaveName = $derived(
    !renaming &&
      exam !== null &&
      examNameSchema.safeParse(nameDraft.trim()).success &&
      nameDraft.trim() !== exam.name,
  );

  // Load on mount and reload when navigating between exam ids without an
  // unmount (the dynamic param changes but the component instance persists).
  $effect(() => {
    void examId;
    void load();
  });

  async function load(): Promise<void> {
    loadFailed = null;
    try {
      const e = await getExamById(examId);
      if (!e) {
        loadFailed = mn.exams.detail.notFound;
        exam = null;
        return;
      }
      exam = e;
      nameDraft = e.name;
      keys = await listAnswerKeysByExam(examId);
      const tpl = await getTemplate(e.template_id);
      templateTitle = tpl?.title ?? "";
    } catch (err) {
      loadFailed = String(err);
    }
  }

  async function saveName(): Promise<void> {
    if (!exam || !canSaveName) return;
    renaming = true;
    try {
      await updateExam(exam.id, { name: nameDraft.trim() });
      exam = { ...exam, name: nameDraft.trim() };
      toast.success(mn.exams.toasts.updated);
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      renaming = false;
    }
  }

  function openAdd(): void {
    editingId = null;
    variantDraft = "";
    answerMode = "paste";
    answerJson = "";
    pickedPath = null;
    formOpen = true;
  }

  function openEdit(rec: AnswerKeyRecord): void {
    editingId = rec.id;
    variantDraft = rec.variant;
    answerMode = "paste";
    answerJson = JSON.stringify(rec.answers, null, 2);
    pickedPath = null;
    formOpen = true;
  }

  function cancelForm(): void {
    formOpen = false;
  }

  async function pickAnswerFile(): Promise<void> {
    try {
      const picked = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (typeof picked !== "string") return;
      pickedPath = picked;
      answerJson = await readTextFile(picked);
    } catch (e) {
      toast.error(mn.grade.answerKey.readFailed, { description: String(e) });
    }
  }

  function validateAnswers(): {
    ok: boolean;
    answers?: AnswerKeyEntry[];
    message?: string;
  } {
    if (answerJson.trim() === "") {
      return { ok: false, message: mn.grade.errors.answerKeyRequired };
    }
    try {
      const parsed = JSON.parse(answerJson) as unknown;
      const answers = answersSchema.parse(parsed);
      return { ok: true, answers };
    } catch (e) {
      return {
        ok: false,
        message: e instanceof Error ? e.message : mn.grade.errors.jsonInvalid,
      };
    }
  }

  async function saveVariant(): Promise<void> {
    if (!exam) return;
    const variant = variantDraft.trim();
    if (variant === "") {
      toast.error(mn.exams.detail.validateFailed, {
        description: mn.exams.detail.variantLabel,
      });
      return;
    }
    const validation = validateAnswers();
    if (!validation.ok || !validation.answers) {
      toast.error(mn.exams.detail.validateFailed, {
        description: validation.message,
      });
      return;
    }

    savingVariant = true;
    try {
      // upsert keys on (exam_id, variant); editing the same variant simply
      // overwrites, while renaming a variant creates a new row + leaves the old
      // one, so delete the prior row when its label changed during an edit.
      await upsertAnswerKey({
        exam_id: exam.id,
        variant,
        answers: validation.answers,
      });
      if (editingId !== null) {
        const prior = keys.find((k) => k.id === editingId);
        if (prior && prior.variant !== variant) {
          await deleteAnswerKey(prior.id);
        }
      }
      toast.success(mn.exams.toasts.variantSaved);
      formOpen = false;
      keys = await listAnswerKeysByExam(exam.id);
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      savingVariant = false;
    }
  }

  async function confirmDeleteVariant(): Promise<void> {
    if (!deleteTarget || !exam) return;
    deletingVariant = true;
    try {
      await deleteAnswerKey(deleteTarget.id);
      toast.success(mn.exams.toasts.variantDeleted);
      deleteTarget = null;
      keys = await listAnswerKeysByExam(exam.id);
    } catch (e) {
      toast.error(mn.exams.toasts.saveFailed, { description: String(e) });
    } finally {
      deletingVariant = false;
    }
  }

  function formatAnswerCount(count: number): string {
    return mn.exams.detail.answerCount.replace("{count}", String(count));
  }
</script>

{#if loadFailed}
  <section class="p-8">
    <Card.Root class="border-destructive/40 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-destructive text-base">{loadFailed}</Card.Title>
      </Card.Header>
    </Card.Root>
  </section>
{:else if exam}
  <section class="container mx-auto space-y-6 p-6 lg:p-8">
    <header class="space-y-1">
      <h2 class="text-2xl font-bold tracking-tight">{mn.exams.detail.title}</h2>
    </header>

    <Card.Root class="max-w-2xl">
      <Card.Content class="space-y-4 pt-6">
        <div class="space-y-1.5">
          <Label for="exam-name">{mn.exams.detail.nameLabel}</Label>
          <div class="flex gap-2">
            <Input id="exam-name" bind:value={nameDraft} class="flex-1" />
            <Button onclick={saveName} disabled={!canSaveName}>
              <SaveIcon />
              {mn.exams.detail.rename}
            </Button>
          </div>
        </div>
        <div class="space-y-1.5">
          <Label>{mn.exams.detail.templateLabel}</Label>
          <p class="text-muted-foreground text-sm">{templateTitle}</p>
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root class="max-w-2xl">
      <Card.Header class="flex flex-row items-center justify-between gap-4">
        <div class="space-y-1">
          <Card.Title class="text-base">{mn.exams.detail.variantsTitle}</Card.Title>
          <Card.Description class="text-xs">
            {mn.exams.detail.variantsHint}
          </Card.Description>
        </div>
        <Button size="sm" onclick={openAdd}>
          <PlusIcon />
          {mn.exams.detail.addVariant}
        </Button>
      </Card.Header>
      <Card.Content class="space-y-2">
        {#if keys.length === 0}
          <p class="text-muted-foreground text-sm">{mn.exams.empty}</p>
        {:else}
          <ul class="divide-y">
            {#each keys as key (key.id)}
              <li class="flex items-center justify-between gap-3 py-2">
                <div class="flex items-center gap-3">
                  <Badge variant="secondary">{key.variant}</Badge>
                  <span class="text-muted-foreground text-sm">
                    {formatAnswerCount(key.answers.length)}
                  </span>
                </div>
                <div class="flex gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={() => openEdit(key)}
                  >
                    {mn.exams.detail.edit}
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onclick={() => (deleteTarget = key)}
                    aria-label={mn.exams.detail.deleteVariant}
                  >
                    <Trash2Icon class="text-destructive size-4" />
                  </Button>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      </Card.Content>
    </Card.Root>
  </section>
{/if}

<Dialog.Root
  open={formOpen}
  onOpenChange={(o) => {
    if (!o) formOpen = false;
  }}
>
  <Dialog.Content class="max-w-xl">
    <Dialog.Header>
      <Dialog.Title>{mn.exams.detail.addVariant}</Dialog.Title>
    </Dialog.Header>
    <div class="space-y-4 py-2">
      <div class="space-y-1.5">
        <Label for="variant-label">{mn.exams.detail.variantLabel}</Label>
        <Input
          id="variant-label"
          bind:value={variantDraft}
          placeholder={mn.exams.detail.variantPlaceholder}
        />
      </div>
      <div class="space-y-1.5">
        <Label>{mn.grade.answerKey.label}</Label>
        <Tabs.Root bind:value={answerMode} class="mt-1">
          <Tabs.List>
            <Tabs.Trigger value="paste">{mn.grade.answerKey.tabPaste}</Tabs.Trigger>
            <Tabs.Trigger value="file">{mn.grade.answerKey.tabFile}</Tabs.Trigger>
          </Tabs.List>
          <Tabs.Content value="paste" class="mt-2">
            <textarea
              bind:value={answerJson}
              placeholder={mn.grade.answerKey.paste}
              rows="8"
              class="border-input bg-background text-foreground focus-visible:ring-ring min-h-40 w-full rounded-md border px-3 py-2 font-mono text-xs shadow-sm focus-visible:ring-1 focus-visible:outline-none"
            ></textarea>
          </Tabs.Content>
          <Tabs.Content value="file" class="mt-2 space-y-2">
            <Button type="button" variant="outline" onclick={pickAnswerFile}>
              <FileJsonIcon />
              {mn.grade.answerKey.pickFile}
            </Button>
            {#if pickedPath}
              <p class="text-muted-foreground text-xs">
                {mn.grade.answerKey.pickedFile}: {pickedPath}
              </p>
            {/if}
          </Tabs.Content>
        </Tabs.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={cancelForm}>
        {mn.exams.detail.cancel}
      </Button>
      <Button onclick={saveVariant} disabled={savingVariant}>
        {mn.exams.detail.save}
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
      <Dialog.Title>{mn.exams.detail.deleteVariantConfirm.title}</Dialog.Title>
      <Dialog.Description>
        {mn.exams.detail.deleteVariantConfirm.body}
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (deleteTarget = null)}>
        {mn.exams.detail.deleteVariantConfirm.cancel}
      </Button>
      <Button
        variant="destructive"
        onclick={confirmDeleteVariant}
        disabled={deletingVariant}
      >
        {mn.exams.detail.deleteVariantConfirm.confirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
