<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import FilePlusIcon from "@lucide/svelte/icons/file-plus";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import SaveIcon from "@lucide/svelte/icons/save";
  import ImageIcon from "@lucide/svelte/icons/image";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import Undo2Icon from "@lucide/svelte/icons/undo-2";
  import Redo2Icon from "@lucide/svelte/icons/redo-2";

  import { mn } from "$lib/i18n";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { editorSelection } from "$lib/stores/editorSelection.svelte";
  import {
    createTemplate,
    getTemplate,
    listTemplates,
    updateTemplate,
  } from "$lib/db/templates";
  import { createEmptyTemplate } from "$lib/types/template";
  import { createMongolianStandardTemplate } from "$lib/templates/mongolianStandard";
  import type { OmrTemplate, TemplateSummary } from "$lib/types/template";
  import { pickImage, pickPdf } from "$lib/picker";
  import {
    importImageBackdrop,
    importPdfBackdrop,
    purgeTemplateAssets,
  } from "$lib/fs/templateAssets";
  import { expand } from "$lib/components/editor/groupLayout";

  let templates = $state<TemplateSummary[]>([]);
  let saveDialogOpen = $state(false);
  let saveDialogTitle = $state("");
  let saveDialogMode = $state<"save" | "saveAs">("save");
  let unsavedDialogOpen = $state(false);
  let pendingNewAction: (() => void) | null = null;

  const canSave = $derived(
    currentTemplate.draft !== null && currentTemplate.backdropPath !== null,
  );

  onMount(async () => {
    templates = await listTemplates();
  });

  async function refreshTemplates() {
    templates = await listTemplates();
  }

  function newDraft(template: OmrTemplate) {
    const uuid = crypto.randomUUID();
    currentTemplate.initDraft(template, uuid);
    editorSelection.clear();
  }

  function startNewWith(factory: () => OmrTemplate) {
    if (currentTemplate.isDirty) {
      pendingNewAction = () => {
        const prevUuid = currentTemplate.draftUuid;
        if (prevUuid) {
          purgeTemplateAssets(prevUuid).catch(() => {});
        }
        newDraft(factory());
      };
      unsavedDialogOpen = true;
    } else {
      newDraft(factory());
    }
  }

  function handleNewStandard() {
    startNewWith(() =>
      createMongolianStandardTemplate({ title: mn.editor.presets.standardDefaultTitle }),
    );
  }

  function handleNewEmpty() {
    startNewWith(() => createEmptyTemplate({ title: mn.editor.untitled }));
  }

  async function handleOpen(id: number) {
    const summary = await getTemplate(id);
    if (summary) {
      currentTemplate.load(summary);
      editorSelection.clear();
    }
  }

  async function ensureDraftUuid(): Promise<string> {
    if (!currentTemplate.draftUuid) {
      currentTemplate.draftUuid = crypto.randomUUID();
    }
    return currentTemplate.draftUuid;
  }

  async function handleImportImage() {
    const path = await pickImage();
    if (!path || !currentTemplate.draft) return;
    try {
      const uuid = await ensureDraftUuid();
      const dest = await importImageBackdrop(path, uuid);
      currentTemplate.backdropPath = dest;
      toast.success(mn.editor.toasts.backdropImported);
    } catch {
      toast.error(mn.editor.toasts.backdropImportFailed);
    }
  }

  async function handleImportPdf() {
    const path = await pickPdf();
    if (!path || !currentTemplate.draft) return;
    try {
      const uuid = await ensureDraftUuid();
      const dest = await importPdfBackdrop(path, uuid);
      currentTemplate.backdropPath = dest;
      toast.success(mn.editor.toasts.backdropImported);
    } catch (e) {
      const code = (e as { code?: string })?.code;
      if (code === "pdfium_unavailable") {
        toast.error(mn.errors.pdfium_unavailable);
      } else {
        toast.error(mn.editor.toasts.backdropImportFailed);
      }
    }
  }

  function openSaveDialog(mode: "save" | "saveAs") {
    if (!currentTemplate.draft || !currentTemplate.backdropPath) return;
    saveDialogMode = mode;
    saveDialogTitle =
      mode === "save" && currentTemplate.loadedFrom
        ? currentTemplate.loadedFrom.title
        : currentTemplate.draft.title || mn.editor.untitled;
    saveDialogOpen = true;
  }

  async function handleSave() {
    if (!currentTemplate.draft || !currentTemplate.backdropPath) return;
    if (saveDialogMode === "save" && currentTemplate.loadedFrom) {
      // Update existing row.
      currentTemplate.draft.title = saveDialogTitle;
      try {
        await updateTemplate(
          currentTemplate.loadedFrom.id,
          currentTemplate.draft,
          currentTemplate.backdropPath,
        );
        const reloaded = await getTemplate(currentTemplate.loadedFrom.id);
        if (reloaded) currentTemplate.load(reloaded);
        await refreshTemplates();
        toast.success(mn.editor.toasts.saved);
      } catch {
        toast.error(mn.editor.toasts.saveFailed);
      }
    } else {
      currentTemplate.draft.title = saveDialogTitle;
      try {
        const newId = await createTemplate(
          currentTemplate.draft,
          currentTemplate.backdropPath,
        );
        const reloaded = await getTemplate(newId);
        if (reloaded) currentTemplate.load(reloaded);
        await refreshTemplates();
        toast.success(mn.editor.toasts.saved);
      } catch {
        toast.error(mn.editor.toasts.saveFailed);
      }
    }
    saveDialogOpen = false;
  }

  function handleAddGroup() {
    if (!currentTemplate.draft) return;
    const id = crypto.randomUUID();
    const layout = {
      origin: { x: 0.5, y: 0.5 },
      direction: "horizontal" as const,
      spacing: 0.05,
      count: 5,
    };
    currentTemplate.draft.groups = [
      ...currentTemplate.draft.groups,
      {
        id,
        kind: "question",
        label: `${mn.editor.groups.defaultLabel} ${currentTemplate.draft.groups.length + 1}`,
        bubbles: expand(layout),
        answer_index: null,
        score: 1,
      },
    ];
    editorSelection.select(id);
  }

  function handleDeleteSelected() {
    if (!currentTemplate.draft || !editorSelection.selectedGroupId) return;
    currentTemplate.draft.groups = currentTemplate.draft.groups.filter(
      (g) => g.id !== editorSelection.selectedGroupId,
    );
    editorSelection.clear();
  }
</script>

<div class="bg-background flex h-12 items-center gap-1 border-b px-2">
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="sm" {...props}>
          <FilePlusIcon class="size-4" />
          {mn.editor.toolbar.new}
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content align="start" class="w-56">
      <DropdownMenu.Item onclick={handleNewStandard}>
        {mn.editor.presets.standard}
      </DropdownMenu.Item>
      <DropdownMenu.Item onclick={handleNewEmpty}>
        {mn.editor.presets.empty}
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>

  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="sm" {...props}>
          <FolderOpenIcon class="size-4" />
          {mn.editor.toolbar.open}
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content align="start" class="max-h-80 w-64 overflow-y-auto">
      {#if templates.length === 0}
        <DropdownMenu.Item disabled>—</DropdownMenu.Item>
      {:else}
        {#each templates as t (t.id)}
          <DropdownMenu.Item onclick={() => handleOpen(t.id)}>
            <span class="flex-1 truncate">{t.title}</span>
            <span class="text-muted-foreground ml-2 font-mono text-[10px]">#{t.id}</span>
          </DropdownMenu.Item>
        {/each}
      {/if}
    </DropdownMenu.Content>
  </DropdownMenu.Root>

  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="sm" {...props}>
          <ImageIcon class="size-4" />
          {mn.editor.toolbar.importBackdrop}
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content align="start">
      <DropdownMenu.Item onclick={handleImportImage}>
        {mn.editor.toolbar.importBackdropImage}
      </DropdownMenu.Item>
      <DropdownMenu.Item onclick={handleImportPdf}>
        {mn.editor.toolbar.importBackdropPdf}
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>

  <Button
    variant="ghost"
    size="sm"
    disabled={!canSave}
    title={canSave ? "" : mn.editor.toolbar.saveDisabledNoBackdrop}
    onclick={() => openSaveDialog("save")}
  >
    <SaveIcon class="size-4" />
    {mn.editor.toolbar.save}
  </Button>

  <Button
    variant="ghost"
    size="sm"
    disabled={!canSave}
    onclick={() => openSaveDialog("saveAs")}
  >
    {mn.editor.toolbar.saveAs}
  </Button>

  <span class="bg-border mx-2 h-6 w-px"></span>

  <Button variant="ghost" size="sm" onclick={handleAddGroup} disabled={!currentTemplate.draft}>
    <PlusIcon class="size-4" />
    {mn.editor.toolbar.addGroup}
  </Button>

  <Button
    variant="ghost"
    size="sm"
    onclick={handleDeleteSelected}
    disabled={!editorSelection.selectedGroupId}
  >
    <Trash2Icon class="size-4" />
    {mn.editor.toolbar.deleteSelected}
  </Button>

  <span class="bg-border mx-2 h-6 w-px"></span>

  <Button variant="ghost" size="sm" disabled title={mn.editor.toolbar.undoRedoDeferredHint}>
    <Undo2Icon class="size-4" />
  </Button>
  <Button variant="ghost" size="sm" disabled title={mn.editor.toolbar.undoRedoDeferredHint}>
    <Redo2Icon class="size-4" />
  </Button>
</div>

<Dialog.Root bind:open={saveDialogOpen}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{mn.dialog.saveAs.title}</Dialog.Title>
      <Dialog.Description>{mn.dialog.saveAs.body}</Dialog.Description>
    </Dialog.Header>
    <div class="space-y-2">
      <Label for="save-title">{mn.editor.inspector.templateTitle}</Label>
      <Input
        id="save-title"
        bind:value={saveDialogTitle}
        oninput={(e) => (saveDialogTitle = (e.target as HTMLInputElement).value)}
      />
    </div>
    <Dialog.Footer>
      <Button variant="ghost" onclick={() => (saveDialogOpen = false)}>
        {mn.dialog.saveAs.cancel}
      </Button>
      <Button onclick={handleSave}>{mn.dialog.saveAs.confirm}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={unsavedDialogOpen}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{mn.dialog.unsavedChanges.title}</Dialog.Title>
      <Dialog.Description>{mn.dialog.unsavedChanges.body}</Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="ghost" onclick={() => (unsavedDialogOpen = false)}>
        {mn.dialog.unsavedChanges.cancel}
      </Button>
      <Button
        onclick={() => {
          unsavedDialogOpen = false;
          if (pendingNewAction) {
            pendingNewAction();
            pendingNewAction = null;
          }
        }}
      >
        {mn.dialog.unsavedChanges.confirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
