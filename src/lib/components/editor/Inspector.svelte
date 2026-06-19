<script lang="ts">
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { editorSelection } from "$lib/stores/editorSelection.svelte";
  import { mn } from "$lib/i18n";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { expand, infer, type GroupLayout } from "$lib/components/editor/groupLayout";
  import type { BubbleKind } from "$lib/types/template";

  const selectedIndex = $derived.by(() => {
    if (!currentTemplate.draft || !editorSelection.selectedGroupId) return -1;
    return currentTemplate.draft.groups.findIndex(
      (g) => g.id === editorSelection.selectedGroupId,
    );
  });

  const selected = $derived.by(() => {
    if (!currentTemplate.draft || selectedIndex < 0) return null;
    return currentTemplate.draft.groups[selectedIndex];
  });

  const layout = $derived.by<GroupLayout | null>(() =>
    selected ? infer(selected.bubbles) : null,
  );

  function regenerateLayout(next: GroupLayout) {
    if (!currentTemplate.draft || selectedIndex < 0) return;
    currentTemplate.draft.groups[selectedIndex].bubbles = expand(next);
  }

  function setCount(value: number) {
    if (!layout) return;
    const count = Math.max(1, Math.floor(value));
    regenerateLayout({ ...layout, count });
  }

  function setSpacing(value: number) {
    if (!layout) return;
    const spacing = Math.max(0.005, Math.min(0.5, value));
    regenerateLayout({ ...layout, spacing });
  }

  function setDirection(direction: "horizontal" | "vertical") {
    if (!layout) return;
    regenerateLayout({ ...layout, direction });
  }
</script>

<aside class="flex h-full flex-col gap-4 overflow-y-auto p-4 text-sm">
  {#if !selected || !currentTemplate.draft}
    <section class="space-y-2">
      <Label for="tpl-title">{mn.editor.inspector.templateTitle}</Label>
      {#if currentTemplate.draft}
        <Input
          id="tpl-title"
          value={currentTemplate.draft.title}
          oninput={(e) => {
            if (currentTemplate.draft) {
              currentTemplate.draft.title = (e.target as HTMLInputElement).value;
            }
          }}
        />
      {/if}
      {#if !selected}
        <p class="text-muted-foreground pt-2 text-xs">{mn.editor.inspector.noSelection}</p>
      {/if}
    </section>
  {:else}
    <section class="space-y-2">
      <Label for="grp-label">{mn.editor.inspector.groupLabel}</Label>
      <Input
        id="grp-label"
        value={selected.label}
        oninput={(e) => {
          if (currentTemplate.draft && selectedIndex >= 0) {
            currentTemplate.draft.groups[selectedIndex].label = (
              e.target as HTMLInputElement
            ).value;
          }
        }}
      />
    </section>

    <section class="space-y-2">
      <Label for="grp-kind">{mn.editor.inspector.groupKind}</Label>
      <select
        id="grp-kind"
        class="border-input bg-background h-9 w-full rounded-md border px-3 py-1 text-sm"
        value={selected.kind}
        onchange={(e) => {
          if (currentTemplate.draft && selectedIndex >= 0) {
            currentTemplate.draft.groups[selectedIndex].kind = (
              e.target as HTMLSelectElement
            ).value as BubbleKind;
          }
        }}
      >
        <option value="question">{mn.editor.groups.kindQuestion}</option>
        <option value="student_id">{mn.editor.groups.kindStudentId}</option>
        <option value="variant">{mn.editor.groups.kindVariant}</option>
      </select>
    </section>

    {#if layout}
      <section class="space-y-2">
        <Label for="grp-count">{mn.editor.inspector.bubbleCount}</Label>
        <Input
          id="grp-count"
          type="number"
          min="1"
          max="20"
          value={layout.count}
          oninput={(e) => setCount(Number((e.target as HTMLInputElement).value))}
        />
      </section>

      <section class="space-y-2">
        <Label>{mn.editor.inspector.direction}</Label>
        <div class="flex gap-2">
          <button
            type="button"
            class="flex-1 rounded border px-2 py-1 text-xs"
            class:bg-accent={layout.direction === "horizontal"}
            onclick={() => setDirection("horizontal")}
          >
            {mn.editor.inspector.directionHorizontal}
          </button>
          <button
            type="button"
            class="flex-1 rounded border px-2 py-1 text-xs"
            class:bg-accent={layout.direction === "vertical"}
            onclick={() => setDirection("vertical")}
          >
            {mn.editor.inspector.directionVertical}
          </button>
        </div>
      </section>

      <section class="space-y-2">
        <Label for="grp-spacing">
          {mn.editor.inspector.spacing}
          <span class="text-muted-foreground ml-2 font-mono text-xs">
            {layout.spacing.toFixed(3)}
          </span>
        </Label>
        <input
          id="grp-spacing"
          type="range"
          min="0.005"
          max="0.2"
          step="0.005"
          value={layout.spacing}
          class="w-full"
          oninput={(e) => setSpacing(Number((e.target as HTMLInputElement).value))}
        />
      </section>
    {:else}
      <p
        class="bg-muted text-muted-foreground rounded px-2 py-1 text-xs"
      >
        {mn.editor.inspector.manualLayoutBadge}
      </p>
    {/if}

    {#if selected.kind === "question"}
      <section class="space-y-2">
        <Label for="grp-answer">
          {mn.editor.inspector.answerIndex}
          <span class="text-muted-foreground ml-2 text-xs">
            {mn.editor.inspector.answerIndexHint}
          </span>
        </Label>
        <Input
          id="grp-answer"
          type="number"
          min="0"
          max={Math.max(0, selected.bubbles.length - 1)}
          value={selected.answer_index ?? ""}
          oninput={(e) => {
            if (!currentTemplate.draft || selectedIndex < 0) return;
            const v = (e.target as HTMLInputElement).value;
            currentTemplate.draft.groups[selectedIndex].answer_index =
              v === "" ? null : Math.max(0, Math.floor(Number(v)));
          }}
        />
      </section>
    {/if}

    <section class="space-y-2">
      <Label for="grp-score">{mn.editor.inspector.score}</Label>
      <Input
        id="grp-score"
        type="number"
        min="0"
        step="0.5"
        value={selected.score}
        oninput={(e) => {
          if (!currentTemplate.draft || selectedIndex < 0) return;
          currentTemplate.draft.groups[selectedIndex].score = Math.max(
            0,
            Number((e.target as HTMLInputElement).value),
          );
        }}
      />
    </section>
  {/if}
</aside>
