<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import ClipboardListIcon from "@lucide/svelte/icons/clipboard-list";

  import { mn } from "$lib/i18n";
  import { listRecentResults, type RecentResultRow } from "$lib/db/results";

  let rows = $state<RecentResultRow[] | null>(null);

  onMount(async () => {
    try {
      rows = await listRecentResults(5);
    } catch (e) {
      console.error("recent results fetch failed", e);
      toast.error(mn.errors.unknown);
      rows = [];
    }
  });

  function formatDate(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleDateString();
  }
</script>

<Card.Root class="h-full">
  <Card.Header class="flex flex-row items-center justify-between gap-2">
    <Card.Title class="text-base">{mn.dashboard.recentResults.title}</Card.Title>
    {#if rows && rows.length > 0}
      <a href="/results" class="text-primary text-xs hover:underline">
        {mn.dashboard.recentResults.viewAll}
      </a>
    {/if}
  </Card.Header>
  <Card.Content>
    {#if rows === null}
      <div class="space-y-2">
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-full" />
        <Skeleton class="h-6 w-4/6" />
      </div>
    {:else if rows.length === 0}
      <div class="flex flex-col items-start gap-2 py-4">
        <ClipboardListIcon class="text-muted-foreground size-6" />
        <p class="text-muted-foreground text-sm">{mn.dashboard.recentResults.empty}</p>
      </div>
    {:else}
      <table class="w-full text-sm">
        <thead class="text-muted-foreground text-xs">
          <tr class="border-b">
            <th class="py-1.5 text-left font-normal">
              {mn.dashboard.recentResults.column.student}
            </th>
            <th class="py-1.5 text-right font-normal">
              {mn.dashboard.recentResults.column.score}
            </th>
            <th class="py-1.5 text-right font-normal">
              {mn.dashboard.recentResults.column.date}
            </th>
          </tr>
        </thead>
        <tbody>
          {#each rows as r (r.id)}
            <tr class="border-b last:border-0">
              <td class="py-1.5 font-mono text-xs">{r.student_id ?? "—"}</td>
              <td class="py-1.5 text-right">{r.total_score.toFixed(1)}</td>
              <td class="text-muted-foreground py-1.5 text-right text-xs">
                {formatDate(r.created_at)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Card.Content>
</Card.Root>
