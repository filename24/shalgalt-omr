<script lang="ts">
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import FilePlusIcon from "@lucide/svelte/icons/file-plus";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import ScanLineIcon from "@lucide/svelte/icons/scan-line";

  import { mn } from "$lib/i18n";

  function openProject() {
    // Project-file format lands with #P4-01..03; for P2 the action is a
    // graceful "coming soon" toast rather than a broken file dialog.
    toast.info(mn.dashboard.actions.openProjectComingSoon);
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
