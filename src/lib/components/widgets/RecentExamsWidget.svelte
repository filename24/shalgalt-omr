<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Badge } from "$lib/components/ui/badge";
  import FilePlusIcon from "@lucide/svelte/icons/file-plus";

  import { mn } from "$lib/i18n";
  import { listExams } from "$lib/db/exams";
  import type { ExamSummary } from "$lib/types/exam";

  let exams = $state<ExamSummary[] | null>(null);

  onMount(async () => {
    try {
      const all = await listExams();
      exams = all.slice(0, 5);
    } catch (e) {
      console.error("recent exams fetch failed", e);
      toast.error(mn.errors.unknown);
      exams = [];
    }
  });
</script>

<Card.Root class="h-full">
  <Card.Header>
    <Card.Title class="text-base">{mn.dashboard.recentExams.title}</Card.Title>
  </Card.Header>
  <Card.Content>
    {#if exams === null}
      <div class="space-y-2">
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-5/6" />
        <Skeleton class="h-6 w-4/6" />
      </div>
    {:else if exams.length === 0}
      <div class="flex flex-col items-start gap-3 py-4">
        <FilePlusIcon class="text-muted-foreground size-6" />
        <p class="text-muted-foreground text-sm">{mn.dashboard.recentExams.empty}</p>
        <Button variant="outline" size="sm" onclick={() => goto("/exams")}>
          {mn.dashboard.recentExams.cta}
        </Button>
      </div>
    {:else}
      <ul class="space-y-1">
        {#each exams as exam (exam.id)}
          <li>
            <a
              href={`/exams/${exam.id}`}
              class="hover:bg-muted flex items-center justify-between gap-2 rounded-md px-2 py-1.5 text-sm"
            >
              <span class="truncate">{exam.name}</span>
              <Badge variant="secondary">{exam.variant_count}</Badge>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </Card.Content>
</Card.Root>
