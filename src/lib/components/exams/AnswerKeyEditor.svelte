<script lang="ts">
  /**
   * Per-question option-button answer-key editor (P4-05).
   *
   * Replaces the JSON paste/file flow on the exam-detail page. Renders one row
   * per `kind === "question"` group with a horizontal set of toggle buttons —
   * one per option index — and maintains a multi-correct selection.
   *
   * `value` is `$bindable()` so the parent can two-way bind it; it is rebuilt as
   * one `AnswerKeyEntry` per question group (including groups with an empty
   * selection) so the consumer can detect incompleteness before persisting.
   */
  import { mn } from "$lib/i18n";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import type { BubbleGroup } from "$lib/types/template";
  import type { AnswerKeyEntry } from "$lib/types/generated/AnswerKeyEntry";

  interface Props {
    groups: BubbleGroup[];
    value: AnswerKeyEntry[];
    onChange?: (entries: AnswerKeyEntry[]) => void;
  }

  let { groups, value = $bindable(), onChange }: Props = $props();

  // Only question groups carry an answer key; preserve template order.
  const questionGroups = $derived(groups.filter((g) => g.kind === "question"));

  // Internal selection: group.id -> Set<number> of selected option indices.
  // A plain reassigned object keeps Svelte 5 reactivity simple (we replace the
  // whole map on every mutation rather than mutating Sets in place).
  let selection = $state<Record<string, Set<number>>>({});
  // Per-question points: group.id -> score. Seeded from the stored key's score
  // when present, else the template's `BubbleGroup.score` default.
  let scores = $state<Record<string, number>>({});

  // Initialize internal selection from the incoming `value`. Runs whenever the
  // parent assigns a new `value` (e.g. opening the edit dialog for a variant) or
  // the group set changes. We seed every question group so the map is complete.
  $effect(() => {
    const nextSel: Record<string, Set<number>> = {};
    const nextScores: Record<string, number> = {};
    for (const g of questionGroups) {
      nextSel[g.id] = new Set<number>();
      nextScores[g.id] = g.score;
    }
    for (const entry of value) {
      if (entry.group_id in nextSel) {
        nextSel[entry.group_id] = new Set<number>(entry.correct_indices);
        if (entry.score != null) nextScores[entry.group_id] = entry.score;
      }
    }
    selection = nextSel;
    scores = nextScores;
  });

  /**
   * Rebuild `value` from the current internal selection: one entry per question
   * group, indices sorted ascending and the question's points. Empty selections
   * are intentionally kept so the parent can flag incomplete answer keys.
   */
  function syncValue(
    nextSel: Record<string, Set<number>>,
    nextScores: Record<string, number>,
  ): void {
    const entries: AnswerKeyEntry[] = questionGroups.map((g) => ({
      group_id: g.id,
      correct_indices: Array.from(nextSel[g.id] ?? new Set<number>()).sort(
        (a, b) => a - b,
      ),
      score: nextScores[g.id] ?? g.score,
    }));
    value = entries;
    onChange?.(entries);
  }

  function toggle(groupId: string, index: number): void {
    const current = selection[groupId] ?? new Set<number>();
    const updated = new Set(current);
    if (updated.has(index)) {
      updated.delete(index);
    } else {
      updated.add(index);
    }
    const next = { ...selection, [groupId]: updated };
    selection = next;
    syncValue(next, scores);
  }

  function setScore(groupId: string, value: number): void {
    const next = { ...scores, [groupId]: Math.max(0, value) };
    scores = next;
    syncValue(selection, next);
  }

  function isSelected(groupId: string, index: number): boolean {
    return selection[groupId]?.has(index) ?? false;
  }

  function isEmpty(groupId: string): boolean {
    return (selection[groupId]?.size ?? 0) === 0;
  }

  /**
   * Seed every question group's selection from the template's canonical
   * `answer_index`, overwriting any current selection. Groups with a null
   * `answer_index` are cleared.
   */
  function seedFromTemplate(): void {
    const next: Record<string, Set<number>> = {};
    for (const g of questionGroups) {
      next[g.id] =
        g.answer_index !== null
          ? new Set<number>([g.answer_index])
          : new Set<number>();
    }
    selection = next;
    syncValue(next, scores);
  }

  // Prefer "section · label" when a section grouping is present.
  function rowLabel(g: BubbleGroup): string {
    return g.section ? `${g.section} · ${g.label}` : g.label;
  }

  // Questions with at least one selected option count as "in this exam". Unselected
  // questions are excluded (the grading engine skips groups with no answer key).
  const selectedCount = $derived(
    questionGroups.filter((g) => !isEmpty(g.id)).length,
  );
  const countLabel = $derived(
    mn.exams.answerKey.selectedCount.replace("{count}", String(selectedCount)),
  );
</script>

{#if questionGroups.length === 0}
  <p class="text-muted-foreground text-sm">{mn.exams.answerKey.noQuestions}</p>
{:else}
  <div class="space-y-3">
    <div class="flex items-start justify-between gap-3">
      <div class="space-y-1">
        <p class="text-muted-foreground text-sm">
          {mn.exams.answerKey.optionsHeading}
        </p>
        <p class="text-muted-foreground/80 text-xs">
          {mn.exams.answerKey.optionsHint}
        </p>
        <p class="text-foreground text-xs font-medium">{countLabel}</p>
      </div>
      <Button type="button" variant="outline" size="sm" onclick={seedFromTemplate}>
        {mn.exams.answerKey.seedFromTemplate}
      </Button>
    </div>

    <ul class="max-h-80 space-y-2 overflow-y-auto pr-1">
      {#each questionGroups as group (group.id)}
        <li class="flex flex-wrap items-center gap-x-3 gap-y-1.5">
          <span class="text-foreground min-w-28 flex-1 truncate text-sm font-medium">
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
          <label class="text-muted-foreground flex items-center gap-1.5 text-xs">
            {mn.exams.answerKey.points}
            <Input
              type="number"
              min="0"
              step="0.5"
              class="h-8 w-16"
              value={scores[group.id] ?? group.score}
              oninput={(e) =>
                setScore(group.id, Number((e.target as HTMLInputElement).value))}
            />
          </label>
          {#if isEmpty(group.id)}
            <span class="text-muted-foreground/70 text-xs italic">
              {mn.exams.answerKey.excluded}
            </span>
          {/if}
        </li>
      {/each}
    </ul>
  </div>
{/if}
