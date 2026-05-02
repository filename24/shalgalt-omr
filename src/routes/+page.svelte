<script lang="ts">
  import { listTemplates } from "$lib/db/templates";
  import type { TemplateSummary } from "$lib/types/template";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import { toast } from "svelte-sonner";

  let templates = $state<TemplateSummary[] | null>(null);
  let loading = $state(false);

  async function refresh() {
    loading = true;
    try {
      templates = await listTemplates();
      toast.success("Templates loaded", {
        description: `${templates.length} template(s) found.`,
      });
    } catch (e) {
      toast.error("Failed to load templates", { description: String(e) });
    } finally {
      loading = false;
    }
  }
</script>

<section class="p-8">
  <header class="mb-6">
    <h2 class="text-2xl font-bold">Dashboard</h2>
    <p class="text-muted-foreground text-sm">
      Local-First OMR Grading IDE — Phase 0 foundation.
    </p>
  </header>

  <div class="mb-6 flex items-center gap-3">
    <Button onclick={refresh} disabled={loading}>
      {loading ? "Loading..." : "Refresh templates"}
    </Button>
    {#if templates}
      <Badge variant="secondary">{templates.length} loaded</Badge>
    {/if}
  </div>

  {#if templates}
    <ul class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {#each templates as t (t.id)}
        <li>
          <Card.Root>
            <Card.Header>
              <Card.Title>{t.title}</Card.Title>
              <Card.Description>Template #{t.id}</Card.Description>
            </Card.Header>
          </Card.Root>
        </li>
      {:else}
        <li class="text-muted-foreground text-sm">No templates saved yet.</li>
      {/each}
    </ul>
  {/if}
</section>
