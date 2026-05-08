<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import FilePlusIcon from "@lucide/svelte/icons/file-plus";

  import { mn } from "$lib/i18n";
  import { listTemplates } from "$lib/db/templates";
  import type { TemplateSummary } from "$lib/types/template";

  // The dedicated `exams` table arrives with #P4-04. Until then the closest
  // proxy is the templates list — recent templates approximate "recent
  // exams" without forcing a placeholder migration.
  let templates = $state<TemplateSummary[] | null>(null);

  onMount(async () => {
    try {
      const all = await listTemplates();
      templates = all.slice(0, 5);
    } catch (e) {
      console.error("recent templates fetch failed", e);
      toast.error(mn.errors.unknown);
      templates = [];
    }
  });
</script>

<Card.Root class="h-full">
  <Card.Header>
    <Card.Title class="text-base">{mn.dashboard.recentExams.title}</Card.Title>
  </Card.Header>
  <Card.Content>
    {#if templates === null}
      <div class="space-y-2">
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-5/6" />
        <Skeleton class="h-6 w-4/6" />
      </div>
    {:else if templates.length === 0}
      <div class="flex flex-col items-start gap-3 py-4">
        <FilePlusIcon class="text-muted-foreground size-6" />
        <p class="text-muted-foreground text-sm">{mn.dashboard.recentExams.empty}</p>
        <Button variant="outline" size="sm" onclick={() => goto("/editor")}>
          {mn.dashboard.recentExams.cta}
        </Button>
      </div>
    {:else}
      <ul class="space-y-1">
        {#each templates as t (t.id)}
          <li>
            <a
              href="/editor"
              class="hover:bg-muted flex items-center justify-between rounded-md px-2 py-1.5 text-sm"
            >
              <span class="truncate">{t.title || mn.editor.untitled}</span>
              <span class="text-muted-foreground ml-2 font-mono text-[10px]">#{t.id}</span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </Card.Content>
</Card.Root>
