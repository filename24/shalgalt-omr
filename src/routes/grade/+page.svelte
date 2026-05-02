<script lang="ts">
  import { progress } from "$lib/stores/progress.svelte";
  import { session } from "$lib/stores/session.svelte";
  import { gradePdf } from "$lib/ipc/scan";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import { toast } from "svelte-sonner";

  let pdfPath = $state(session.lastPdfPath ?? "");
  let templateId = $state(session.lastOpenedTemplateId ?? 1);
  let job = $state<string | null>(session.activeJobId);
  let busy = $state(false);

  const last = $derived(progress.last);
  const pct = $derived(
    last && last.total > 0 ? Math.round((last.processed / last.total) * 100) : null,
  );

  async function start() {
    busy = true;
    try {
      const r = await gradePdf(pdfPath, templateId);
      job = r.task_id;
      session.setActiveJob(r.task_id);
      session.recordPdfPath(pdfPath);
      session.recordTemplate(templateId);
      toast.success("Grading started", { description: `task_id ${r.task_id}` });
    } catch (e) {
      toast.error("Failed to start grading", { description: String(e) });
    } finally {
      busy = false;
    }
  }
</script>

<section class="p-8">
  <header class="mb-6">
    <h2 class="text-2xl font-bold">Grade PDF</h2>
    <p class="text-muted-foreground text-sm">
      Rule 1 — pass an <strong>absolute file path</strong> instead of attaching the file.
    </p>
  </header>

  <Card.Root class="max-w-2xl">
    <Card.Header>
      <Card.Title>Start a grade job</Card.Title>
      <Card.Description>
        Spawns a background tokio task; progress streams back via the
        <code>task-progress</code> event.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-4">
      <div class="space-y-1.5">
        <Label for="pdf-path">PDF absolute path</Label>
        <Input
          id="pdf-path"
          bind:value={pdfPath}
          placeholder="/Users/.../scans/2026-05-01.pdf"
        />
      </div>
      <div class="space-y-1.5">
        <Label for="template-id">Template ID</Label>
        <Input id="template-id" type="number" bind:value={templateId} class="w-32" />
      </div>
    </Card.Content>
    <Card.Footer class="flex items-center justify-between">
      <Button onclick={start} disabled={busy || !pdfPath}>
        {busy ? "Starting..." : "Start grading"}
      </Button>
      {#if job}
        <Badge variant="secondary">task_id {job.slice(0, 8)}</Badge>
      {/if}
    </Card.Footer>
  </Card.Root>

  {#if last}
    <Card.Root class="mt-6 max-w-2xl">
      <Card.Header>
        <Card.Title class="text-base">Live progress</Card.Title>
      </Card.Header>
      <Card.Content class="text-sm">
        <p>
          stage = <code>{last.stage}</code>
          ({last.processed} / {last.total}{pct !== null ? ` — ${pct}%` : ""})
        </p>
        {#if last.message}
          <p class="text-muted-foreground mt-1">{last.message}</p>
        {/if}
      </Card.Content>
    </Card.Root>
  {/if}
</section>
