<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
</script>

<section class="p-8">
  <header class="mb-6 flex items-center gap-3">
    <h2 class="text-2xl font-bold">Results</h2>
    <Badge variant="outline">P3 / P4</Badge>
  </header>

  <Card.Root class="max-w-2xl">
    <Card.Header>
      <Card.Title>Result browsing pending</Card.Title>
      <Card.Description>
        Phase 3 adds the result table with click-to-preview and the manual correction UI;
        Phase 4 wires up the rust_xlsxwriter export and read-only axum endpoints.
      </Card.Description>
    </Card.Header>
    <Card.Content class="text-muted-foreground text-sm">
      <ul class="list-disc space-y-1 pl-5">
        <li>Per-template result grid sourced from <code>$lib/db/results</code></li>
        <li>Detail panel rendering the graded image via <code>asset://localhost/</code></li>
        <li>Export-to-xlsx action backed by the <code>export_results_xlsx</code> command</li>
      </ul>
    </Card.Content>
  </Card.Root>
</section>
