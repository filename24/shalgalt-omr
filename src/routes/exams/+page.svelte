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

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

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
    <Button onclick={openCreate}>
      <PlusIcon />
      {mn.exams.newExam}
    </Button>
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
