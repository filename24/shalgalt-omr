<script lang="ts">
  /**
   * Dashboard entry point into the manual-review flow.
   *
   * Without this widget the only way to reach `/review/[job_id]` is the
   * post-grade CTA on `/grade`, which disappears the moment the user
   * navigates away. Listing recent jobs here makes review re-entry possible
   * after a refresh or app restart.
   */
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";

  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Badge } from "$lib/components/ui/badge";
  import ScanLineIcon from "@lucide/svelte/icons/scan-line";

  import { mn } from "$lib/i18n";
  import { listRecentJobs } from "$lib/db/jobs";
  import type { Job, JobStatus } from "$lib/types/job";

  let rows = $state<Job[] | null>(null);

  onMount(async () => {
    try {
      rows = await listRecentJobs(5);
    } catch (e) {
      console.error("recent jobs fetch failed", e);
      toast.error(mn.errors.unknown);
      rows = [];
    }
  });

  function pdfBaseName(p: string): string {
    const i = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
    return i >= 0 ? p.slice(i + 1) : p;
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleDateString();
  }

  function statusVariant(
    s: JobStatus,
  ): "default" | "secondary" | "destructive" | "outline" {
    switch (s) {
      case "done":
        return "secondary";
      case "failed":
      case "canceled":
        return "destructive";
      case "running":
      case "queued":
        return "default";
    }
  }

  function rowIsReviewable(j: Job): boolean {
    return j.status === "done" && j.graded_sheets_json !== null;
  }

  function openJob(j: Job): void {
    if (!rowIsReviewable(j)) return;
    void goto(`/review/${j.id}`);
  }
</script>

<Card.Root class="h-full">
  <Card.Header>
    <Card.Title class="text-base">{mn.dashboard.recentJobs.title}</Card.Title>
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
        <ScanLineIcon class="text-muted-foreground size-6" />
        <p class="text-muted-foreground text-sm">{mn.dashboard.recentJobs.empty}</p>
      </div>
    {:else}
      <table class="w-full text-sm">
        <thead class="text-muted-foreground text-xs">
          <tr class="border-b">
            <th class="py-1.5 text-left font-normal">
              {mn.dashboard.recentJobs.column.pdf}
            </th>
            <th class="py-1.5 text-left font-normal">
              {mn.dashboard.recentJobs.column.status}
            </th>
            <th class="py-1.5 text-right font-normal">
              {mn.dashboard.recentJobs.column.flagged}
            </th>
            <th class="py-1.5 text-right font-normal">
              {mn.dashboard.recentJobs.column.date}
            </th>
          </tr>
        </thead>
        <tbody>
          {#each rows as j (j.id)}
            <tr
              class="border-b last:border-0 data-[reviewable=true]:hover:bg-accent/40 data-[reviewable=true]:cursor-pointer"
              data-reviewable={rowIsReviewable(j)}
              onclick={() => openJob(j)}
              role={rowIsReviewable(j) ? "button" : undefined}
              tabindex={rowIsReviewable(j) ? 0 : -1}
              onkeydown={(e) => {
                if (rowIsReviewable(j) && (e.key === "Enter" || e.key === " ")) {
                  e.preventDefault();
                  openJob(j);
                }
              }}
            >
              <td class="max-w-[12rem] truncate py-1.5 font-mono text-xs" title={j.pdf_path}>
                {pdfBaseName(j.pdf_path)}
              </td>
              <td class="py-1.5">
                <Badge variant={statusVariant(j.status)} class="text-[10px]">
                  {mn.job.status[j.status]}
                </Badge>
              </td>
              <td class="py-1.5 text-right">
                {#if j.needs_review_count > 0}
                  <Badge variant="destructive" class="text-[10px]">
                    {j.needs_review_count}
                  </Badge>
                {:else}
                  <span class="text-muted-foreground">—</span>
                {/if}
              </td>
              <td class="text-muted-foreground py-1.5 text-right text-xs">
                {formatDate(j.created_at)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Card.Content>
</Card.Root>
