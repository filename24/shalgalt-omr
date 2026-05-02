<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
</script>

<section class="p-8">
  <header class="mb-6 flex items-center gap-3">
    <h2 class="text-2xl font-bold">Template Editor</h2>
    <Badge variant="outline">P1</Badge>
  </header>

  <Card.Root class="max-w-2xl">
    <Card.Header>
      <Card.Title>svelte-konva canvas pending</Card.Title>
      <Card.Description>
        Phase 1 lights up the visual editor: 4 corner markers, drag-to-place bubble
        groups, and an answer-key inspector. Templates serialize to
        <code>OmrTemplate</code> JSON (Rule 3) and persist via
        <code>$lib/db/templates</code>.
      </Card.Description>
    </Card.Header>
    <Card.Content class="text-muted-foreground text-sm">
      <ul class="list-disc space-y-1 pl-5">
        <li>Marker layer + perspective preview</li>
        <li>Bubble group tool (student-id / question)</li>
        <li>Answer-key inspector bound to <code>currentTemplate</code> store</li>
      </ul>
    </Card.Content>
  </Card.Root>
</section>
