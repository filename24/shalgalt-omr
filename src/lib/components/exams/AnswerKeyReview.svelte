<script lang="ts">
  /**
   * Side-by-side review of a scanned answer key (P4-06).
   *
   * Left: the rasterized scan preview (loaded via `asset://localhost/`, Rule 1).
   * Right: one row per question group, pre-filled from the CV reading. Groups whose
   * reading landed in the uncertain band are ringed and flagged; Save stays disabled
   * until every uncertain group is reviewed (its selection touched) or explicitly
   * accepted, and until no question is left blank.
   *
   * The reviewer can override any reading before saving. Persistence is the parent's
   * job (it calls the same `upsertAnswerKey` path the manual editor uses).
   */
  import { mn } from "$lib/i18n";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import type { BubbleGroup } from "$lib/types/template";
  import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";
  import type { AnswerKeyImportSummary } from "$lib/ipc/scan";

  interface Props {
    /** Question groups in template order — index-aligned with `summary.answers`. */
    questionGroups: BubbleGroup[];
    summary: AnswerKeyImportSummary;
    /** `asset://` URL for the scan preview image. */
    previewUrl: string;
    saving?: boolean;
    onSave: (entries: AnswerKeyEntry[]) => void;
    onCancel: () => void;
  }

  let {
    questionGroups,
    summary,
    previewUrl,
    saving = false,
    onSave,
    onCancel,
  }: Props = $props();

  // group.id -> Set<number> of selected option indices, seeded from the reading.
  let selection = $state<Record<string, Set<number>>>({});
  // Groups the reviewer has touched or explicitly accepted (clears the uncertain gate).
  let reviewed = $state<Set<string>>(new Set());

  // Group ids the CV flagged as uncertain, resolved from the index-aligned summary.
  const uncertainIds = $derived(
    new Set(summary.uncertain_groups.map((i) => summary.group_ids[i])),
  );

  // Seed selection from the reading whenever a fresh summary arrives. `-1` (blank)
  // becomes an empty set so the reviewer is forced to fill it before saving.
  $effect(() => {
    const next: Record<string, Set<number>> = {};
    questionGroups.forEach((g, i) => {
      const read = summary.answers[i] ?? -1;
      next[g.id] = read >= 0 ? new Set<number>([read]) : new Set<number>();
    });
    selection = next;
    reviewed = new Set();
  });

  function isSelected(groupId: string, index: number): boolean {
    return selection[groupId]?.has(index) ?? false;
  }

  function isEmpty(groupId: string): boolean {
    return (selection[groupId]?.size ?? 0) === 0;
  }

  function isUncertain(groupId: string): boolean {
    return uncertainIds.has(groupId);
  }

  function toggle(groupId: string, index: number): void {
    const current = selection[groupId] ?? new Set<number>();
    const updated = new Set(current);
    if (updated.has(index)) {
      updated.delete(index);
    } else {
      updated.add(index);
    }
    selection = { ...selection, [groupId]: updated };
    // Touching a group counts as reviewing it.
    reviewed = new Set([...reviewed, groupId]);
  }

  /** Accept every uncertain reading as-is, clearing the review gate in one click. */
  function acceptUncertain(): void {
    reviewed = new Set([...reviewed, ...uncertainIds]);
  }

  // A question with at least one selected option is "in this exam". Unselected
  // questions are excluded — the grading engine skips groups absent from the key.
  const answeredCount = $derived(
    questionGroups.filter((g) => !isEmpty(g.id)).length,
  );
  const countLabel = $derived(
    mn.exams.answerKey.selectedCount.replace("{count}", String(answeredCount)),
  );
  // Only block on uncertain readings that are still part of the exam (selected).
  // If the teacher excludes an uncertain question (leaves it blank), its
  // uncertainty no longer matters.
  const pendingUncertain = $derived(
    questionGroups.some(
      (g) => uncertainIds.has(g.id) && !isEmpty(g.id) && !reviewed.has(g.id),
    ),
  );
  const allAccepted = $derived(!pendingUncertain);
  const canSave = $derived(!saving && answeredCount > 0 && !pendingUncertain);

  function rowLabel(g: BubbleGroup): string {
    return g.section ? `${g.section} · ${g.label}` : g.label;
  }

  function save(): void {
    if (!canSave) return;
    // Emit only the questions that are part of the exam (have a selection);
    // unselected ones are excluded and must not reach the key (each entry
    // requires >= 1 correct index).
    const entries: AnswerKeyEntry[] = questionGroups
      .filter((g) => !isEmpty(g.id))
      .map((g) => ({
        group_id: g.id,
        correct_indices: Array.from(selection[g.id] ?? new Set<number>()).sort(
          (a, b) => a - b,
        ),
      }));
    onSave(entries);
  }
</script>

<div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
  <!-- Scan preview -->
  <div class="space-y-2">
    <p class="text-muted-foreground text-sm font-medium">
      {mn.exams.answerKey.scan.previewHeading}
    </p>
    <div class="bg-muted/30 overflow-hidden rounded-lg border">
      <img
        src={previewUrl}
        alt={mn.exams.answerKey.scan.previewHeading}
        class="max-h-[70vh] w-full object-contain"
      />
    </div>
  </div>

  <!-- Read answers -->
  <div class="space-y-3">
    <div class="flex items-start justify-between gap-3">
      <div class="space-y-0.5">
        <p class="text-muted-foreground text-sm font-medium">
          {mn.exams.answerKey.scan.answersHeading}
        </p>
        <p class="text-foreground text-xs font-medium">{countLabel}</p>
      </div>
      {#if uncertainIds.size > 0}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onclick={acceptUncertain}
          disabled={allAccepted}
        >
          {allAccepted
            ? mn.exams.answerKey.scan.accepted
            : mn.exams.answerKey.scan.acceptUncertain}
        </Button>
      {/if}
    </div>

    {#if questionGroups.length === 0}
      <p class="text-muted-foreground text-sm">
        {mn.exams.answerKey.scan.emptyResult}
      </p>
    {:else}
      {#if pendingUncertain}
        <p
          class="text-warning-foreground flex items-center gap-2 rounded-md border border-amber-400/50 bg-amber-50 px-3 py-2 text-xs dark:bg-amber-950/30"
        >
          <TriangleAlertIcon class="size-4 shrink-0 text-amber-500" />
          {mn.exams.answerKey.scan.uncertainNotice}
        </p>
      {/if}

      <ul class="max-h-[60vh] space-y-2 overflow-y-auto pr-1">
        {#each questionGroups as group (group.id)}
          {@const uncertain = isUncertain(group.id)}
          <li
            class="flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md px-2 py-1.5 {uncertain
              ? 'ring-1 ring-amber-400/70'
              : ''}"
          >
            <span
              class="text-foreground flex min-w-28 flex-1 items-center gap-1.5 truncate text-sm font-medium"
            >
              {#if uncertain}
                <TriangleAlertIcon class="size-3.5 shrink-0 text-amber-500" />
              {/if}
              {rowLabel(group)}
            </span>
            <div class="flex flex-wrap gap-1.5">
              {#each Array.from({ length: group.bubbles.length }, (_, i) => i) as optionIndex (optionIndex)}
                {@const selected = isSelected(group.id, optionIndex)}
                <Button
                  type="button"
                  variant={selected ? "default" : "outline"}
                  size="sm"
                  aria-pressed={selected}
                  class="h-8 min-w-9 px-2"
                  onclick={() => toggle(group.id, optionIndex)}
                >
                  {optionIndex + 1}
                </Button>
              {/each}
            </div>
            {#if isEmpty(group.id)}
              <Badge variant="outline" class="text-muted-foreground text-xs">
                {mn.exams.answerKey.excluded}
              </Badge>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="outline" onclick={onCancel} disabled={saving}>
        {mn.exams.answerKey.scan.cancel}
      </Button>
      <Button onclick={save} disabled={!canSave}>
        {mn.exams.answerKey.scan.save}
      </Button>
    </div>
  </div>
</div>
