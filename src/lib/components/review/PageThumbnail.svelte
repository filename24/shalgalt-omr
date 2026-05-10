<script lang="ts">
  /**
   * Compact left-pane card for one page in a grading job.
   * Shows score, page index, and review-state badges. The thumbnail image
   * itself is loaded on demand — for /review's left rail we keep it small
   * and lazy so very large jobs stay snappy.
   */
  import { Badge } from "$lib/components/ui/badge";
  import { assetUrl } from "$lib/fs/templateAssets";
  import { mn } from "$lib/i18n";
  import type { GradedJobSheet } from "$lib/types/job";

  interface Props {
    sheet: GradedJobSheet;
    selected: boolean;
    onSelect: () => void;
  }

  let { sheet, selected, onSelect }: Props = $props();

  const flagged = $derived(sheet.graded.needs_review && !sheet.reviewed);
</script>

<button
  type="button"
  onclick={onSelect}
  class="hover:bg-accent/40 group flex w-full items-center gap-3 rounded-md border p-3 text-left text-sm transition-colors data-[selected=true]:border-primary data-[selected=true]:bg-accent/60"
  data-selected={selected}
>
  <div class="bg-muted h-16 w-12 shrink-0 overflow-hidden rounded border">
    <img
      src={assetUrl(sheet.page_image_path)}
      alt=""
      class="h-full w-full object-cover"
      loading="lazy"
    />
  </div>
  <div class="flex-1">
    <p class="font-medium">
      {mn.review.pageHeader} {sheet.page_index + 1}
    </p>
    <p class="text-muted-foreground text-xs">
      {mn.review.score}: {sheet.graded.total_score.toFixed(1)}
    </p>
    <div class="mt-1 flex gap-1">
      {#if flagged}
        <Badge variant="destructive" class="text-[10px]">
          {mn.review.needsReview}
        </Badge>
      {/if}
      {#if sheet.reviewed}
        <Badge variant="secondary" class="text-[10px]">
          {mn.review.reviewed}
        </Badge>
      {/if}
    </div>
  </div>
</button>
