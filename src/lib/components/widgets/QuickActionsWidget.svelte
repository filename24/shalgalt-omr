<script lang="ts">
  import { goto } from "$app/navigation";

  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import FilePlusIcon from "@lucide/svelte/icons/file-plus";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import ScanLineIcon from "@lucide/svelte/icons/scan-line";

  import { mn } from "$lib/i18n";
  import { pickShalgalt } from "$lib/picker";

  // Pick a `.shalgalt` file and hand it to the shared "Open with" import page,
  // which owns the passphrase prompt + restore flow (same path as the exams
  // list "Open file" button).
  async function openProject(): Promise<void> {
    const path = await pickShalgalt();
    if (!path) return;
    void goto(`/exams/import?path=${encodeURIComponent(path)}`);
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="text-base">{mn.dashboard.actions.title}</Card.Title>
  </Card.Header>
  <Card.Content>
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
      <Button
        variant="default"
        class="h-auto flex-col items-start gap-1 px-4 py-4 text-left"
        onclick={() => goto("/exams")}
      >
        <span class="flex items-center gap-2">
          <FilePlusIcon class="size-5" />
          <span class="text-base font-semibold">{mn.dashboard.actions.newExam}</span>
        </span>
      </Button>
      <Button
        variant="secondary"
        class="h-auto flex-col items-start gap-1 px-4 py-4 text-left"
        onclick={openProject}
      >
        <span class="flex items-center gap-2">
          <FolderOpenIcon class="size-5" />
          <span class="text-base font-semibold">{mn.dashboard.actions.openProject}</span>
        </span>
      </Button>
      <Button
        variant="secondary"
        class="h-auto flex-col items-start gap-1 px-4 py-4 text-left"
        onclick={() => goto("/grade")}
      >
        <span class="flex items-center gap-2">
          <ScanLineIcon class="size-5" />
          <span class="text-base font-semibold">{mn.dashboard.actions.gradePdf}</span>
        </span>
      </Button>
    </div>
  </Card.Content>
</Card.Root>
