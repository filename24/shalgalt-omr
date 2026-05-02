<script lang="ts">
  import { currentTemplate } from "$lib/stores/currentTemplate.svelte";
  import { editorSelection } from "$lib/stores/editorSelection.svelte";
  import { mn } from "$lib/i18n";
  import CircleDotIcon from "@lucide/svelte/icons/circle-dot";
  import CornerDownLeftIcon from "@lucide/svelte/icons/corner-down-left";
  import type { BubbleGroup } from "$lib/types/template";

  const markerLabels = [
    mn.editor.markers.tl,
    mn.editor.markers.tr,
    mn.editor.markers.br,
    mn.editor.markers.bl,
  ];

  const markerColors = ["#ef4444", "#22c55e", "#3b82f6", "#eab308"];

  function kindLabel(kind: "student_id" | "question"): string {
    return kind === "student_id"
      ? mn.editor.groups.kindStudentId
      : mn.editor.groups.kindQuestion;
  }

  // Bucket groups by `section`. Groups without a section share an "other"
  // bucket so nothing is silently hidden when a user creates groups outside
  // of a preset workflow.
  const sectioned = $derived.by(() => {
    if (!currentTemplate.draft) return [] as { name: string; groups: BubbleGroup[] }[];
    const buckets = new Map<string, BubbleGroup[]>();
    const order: string[] = [];
    for (const g of currentTemplate.draft.groups) {
      const key = g.section ?? mn.editor.presets.sections.other;
      if (!buckets.has(key)) {
        buckets.set(key, []);
        order.push(key);
      }
      buckets.get(key)!.push(g);
    }
    return order.map((name) => ({ name, groups: buckets.get(name)! }));
  });
</script>

<aside class="flex h-full flex-col gap-4 overflow-y-auto p-3 text-sm">
  <section>
    <h3 class="text-muted-foreground mb-2 text-xs font-semibold tracking-wide uppercase">
      {mn.editor.layers.markers}
    </h3>
    <ul class="space-y-1">
      {#if currentTemplate.draft}
        {#each currentTemplate.draft.markers as m, i (m.id)}
          <li class="flex items-center gap-2 rounded px-2 py-1">
            <CornerDownLeftIcon class="size-4" style="color: {markerColors[i]}" />
            <span>{markerLabels[i]}</span>
            <span class="text-muted-foreground ml-auto font-mono text-[10px]">
              {m.position.x.toFixed(2)}, {m.position.y.toFixed(2)}
            </span>
          </li>
        {/each}
      {/if}
    </ul>
  </section>

  {#if sectioned.length === 0}
    <section>
      <h3 class="text-muted-foreground mb-2 text-xs font-semibold tracking-wide uppercase">
        {mn.editor.layers.groups}
      </h3>
      <p class="text-muted-foreground text-xs">{mn.editor.layers.empty}</p>
    </section>
  {:else}
    {#each sectioned as bucket (bucket.name)}
      <section>
        <h3
          class="text-muted-foreground mb-2 flex items-center justify-between text-xs font-semibold tracking-wide uppercase"
        >
          <span>{bucket.name}</span>
          <span class="text-muted-foreground/60 font-normal normal-case">
            {bucket.groups.length}
          </span>
        </h3>
        <ul class="space-y-1">
          {#each bucket.groups as g (g.id)}
            {@const selected = editorSelection.selectedGroupId === g.id}
            <li>
              <button
                type="button"
                class="flex w-full items-center gap-2 rounded px-2 py-1 text-left transition-colors"
                class:bg-accent={selected}
                class:text-accent-foreground={selected}
                class:hover:bg-accent={!selected}
                onclick={() => editorSelection.select(g.id)}
              >
                <CircleDotIcon class="size-4" />
                <span class="flex-1 truncate">{g.label || mn.editor.groups.defaultLabel}</span>
                <span
                  class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 font-mono text-[10px]"
                >
                  {kindLabel(g.kind)}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/each}
  {/if}
</aside>
