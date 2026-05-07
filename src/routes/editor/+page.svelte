<script lang="ts">
  import { onMount } from "svelte";
  import { beforeNavigate } from "$app/navigation";
  import EditorShell from "$lib/components/editor/EditorShell.svelte";
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { createMongolianStandardTemplate } from "$lib/templates/mongolianStandard";
  import { mn } from "$lib/i18n";

  // Bootstrap a fresh draft when entering the route with nothing loaded. The
  // Mongolian-standard preset is the friendlier default — most users start
  // from this layout and tweak; advanced users hit "New > Empty" to drop it.
  // We do NOT auto-create one if a template was already opened from the
  // dashboard, because that would clobber the user's work.
  onMount(() => {
    if (!currentTemplate.draft) {
      currentTemplate.initDraft(
        createMongolianStandardTemplate({
          title: mn.editor.presets.standardDefaultTitle,
        }),
        crypto.randomUUID(),
      );
    }
  });

  // SvelteKit in-app navigation guard.
  beforeNavigate(({ cancel }) => {
    if (currentTemplate.isDirty) {
      const confirmed = window.confirm(mn.dialog.unsavedChanges.body);
      if (!confirmed) cancel();
    }
  });

  // Browser-level guard for refresh / window close.
  function handleBeforeUnload(event: BeforeUnloadEvent) {
    if (currentTemplate.isDirty) {
      event.preventDefault();
    }
  }
</script>

<svelte:window onbeforeunload={handleBeforeUnload} />

<div class="h-[calc(100vh-3rem-1.75rem)] w-full">
  <EditorShell />
</div>
