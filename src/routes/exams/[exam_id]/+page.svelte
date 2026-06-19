<script lang="ts">
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";

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
  import { variantNameSchema } from "$lib/schemas/answerKey";
  import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";
  import type { BubbleGroup } from "$lib/types/template";

  import AnswerKeyEditor from "$lib/components/exams/AnswerKeyEditor.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import SaveIcon from "@lucide/svelte/icons/save";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import ScanLineIcon from "@lucide/svelte/icons/scan-line";

  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const examId = $derived(data.examId);

  let exam = $state<Exam | null>(null);
  let templateTitle = $state("");
  // The exam's template groups, loaded once. `null` means the template was
  // deleted out from under the exam (variant add/edit must be disabled).
  let templateGroups = $state<BubbleGroup[] | null>(null);
  let keys = $state<AnswerKeyRecord[]>([]);
  let loadFailed = $state<string | null>(null);

  // Rename state.
  let nameDraft = $state("");
  let renaming = $state(false);

  // Add/edit-variant state. `editingId` null means "adding new".
  let formOpen = $state(false);
  let editingId = $state<number | null>(null);
  let variantDraft = $state("");
  // Bound to the AnswerKeyEditor: one entry per question group (empty selections
  // included so we can detect incompleteness on save).
  let draftAnswers = $state<AnswerKeyEntry[]>([]);
  let savingVariant = $state(false);

  const templateMissing = $derived(templateGroups === null);

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
      templateGroups = tpl ? tpl.schema.groups : null;
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
    if (templateMissing) return;
    editingId = null;
    variantDraft = "";
    // Start empty; the user can seed from template defaults inside the editor.
    draftAnswers = [];
    formOpen = true;
  }

  function openEdit(rec: AnswerKeyRecord): void {
    if (templateMissing) return;
    editingId = rec.id;
    variantDraft = rec.variant;
    // Initialize from the stored answers; the editor fills in any question
    // groups missing from the record as empty selections.
    draftAnswers = rec.answers.map((a) => ({
      group_id: a.group_id,
      correct_indices: [...a.correct_indices],
      score: a.score,
    }));
    formOpen = true;
  }

  function cancelForm(): void {
    formOpen = false;
  }

  async function saveVariant(): Promise<void> {
    if (!exam) return;
    const variant = variantDraft.trim();
    // Variant names must be the single printed letter (A/B/C…) so /grade
    // auto-detection can match a scanned sheet's bubble to this key.
    if (!variantNameSchema.safeParse(variant).success) {
      toast.error(mn.exams.detail.variantInvalid);
      return;
    }

    // The answer key defines which questions are in this exam: keep only the
    // questions the teacher actually answered. Unselected questions are excluded
    // and the grading engine skips them. At least one must be selected.
    const answers = draftAnswers.filter(
      (entry) => entry.correct_indices.length > 0,
    );
    if (answers.length === 0) {
      toast.error(mn.exams.answerKey.noneSelected);
      return;
    }
    let validated: AnswerKeyEntry[];
    try {
      validated = answersSchema.parse(answers);
    } catch (e) {
      toast.error(mn.exams.detail.validateFailed, {
        description: e instanceof Error ? e.message : undefined,
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
        answers: validated,
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

  // Navigate to the scan-to-answer-key screen for one variant (P4-06).
  function gotoScan(v: string): void {
    if (!exam) return;
    void goto(`/exams/${exam.id}/answer-key/${encodeURIComponent(v)}/scan`);
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
          {#if templateMissing}
            <p class="text-destructive text-sm">
              {mn.exams.answerKey.templateMissing}
            </p>
          {:else}
            <p class="text-muted-foreground text-sm">{templateTitle}</p>
          {/if}
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
        <Button size="sm" onclick={openAdd} disabled={templateMissing}>
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
                    onclick={() => gotoScan(key.variant)}
                    disabled={templateMissing}
                  >
                    <ScanLineIcon class="size-4" />
                    {mn.exams.answerKey.scan.action}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={() => openEdit(key)}
                    disabled={templateMissing}
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
        <Label>{mn.exams.answerKey.label}</Label>
        {#if templateGroups}
          <AnswerKeyEditor groups={templateGroups} bind:value={draftAnswers} />
        {/if}
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
