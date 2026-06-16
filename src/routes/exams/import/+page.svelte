<script lang="ts">
  /**
   * "Open with" landing page (P4-07).
   *
   * Reached when the user double-clicks a `.shalgalt` file: the runtime forwards
   * the path here as `?path=…` (see `openWithFile.svelte.ts`). We open the
   * container via `project_open`; if it is encrypted we prompt for the passphrase
   * and retry, then restore its contents into the local DB through the shared
   * `restoreProject` glue and land on the exam list.
   */
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { toast } from "svelte-sonner";

  import { mn } from "$lib/i18n";
  import { projectOpen } from "$lib/ipc/project";
  import { restoreProject } from "$lib/project";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import Loader2Icon from "@lucide/svelte/icons/loader-circle";

  const path = $derived(page.url.searchParams.get("path"));

  let opening = $state(false);
  let needPassphrase = $state(false);
  let passphrase = $state("");
  let failed = $state<string | null>(null);

  // Kick off the open as soon as we have a path. Re-runs only when the path
  // changes; passphrase retries are driven manually from `unlock`.
  $effect(() => {
    const p = path;
    if (p) {
      void attemptOpen();
    } else {
      failed = mn.exams.import.noPath;
    }
  });

  async function attemptOpen(pass?: string): Promise<void> {
    if (!path) return;
    opening = true;
    failed = null;
    try {
      const opened = await projectOpen({ inputPath: path, passphrase: pass });
      await restoreProject(opened);
      toast.success(mn.exams.import.imported, {
        description: opened.manifest.title,
      });
      void goto("/exams");
    } catch (e) {
      const code = (e as { code?: string })?.code;
      if (code === "fileformat.bad_passphrase") {
        // Encrypted (or wrong passphrase): prompt and retry.
        needPassphrase = true;
      } else {
        failed = mn.exams.import.failed;
        toast.error(mn.exams.import.failed, { description: String(e) });
      }
    } finally {
      opening = false;
    }
  }

  function unlock(): void {
    if (!passphrase) return;
    needPassphrase = false;
    void attemptOpen(passphrase);
  }

  function cancel(): void {
    needPassphrase = false;
    void goto("/exams");
  }
</script>

<section class="container mx-auto p-6 lg:p-8">
  <Card.Root class="mx-auto max-w-md">
    <Card.Header>
      <Card.Title class="text-base">{mn.exams.import.title}</Card.Title>
    </Card.Header>
    <Card.Content class="space-y-3">
      {#if failed}
        <p class="text-destructive text-sm">{failed}</p>
        <Button variant="outline" size="sm" onclick={() => goto("/exams")}>
          {mn.exams.import.cancel}
        </Button>
      {:else}
        <p class="text-muted-foreground flex items-center gap-2 text-sm">
          {#if opening}
            <Loader2Icon class="size-4 animate-spin" />
          {/if}
          {mn.exams.import.opening}
        </p>
        {#if path}
          <p class="text-muted-foreground truncate text-xs">{path}</p>
        {/if}
      {/if}
    </Card.Content>
  </Card.Root>
</section>

<Dialog.Root
  open={needPassphrase}
  onOpenChange={(o) => {
    if (!o) cancel();
  }}
>
  <Dialog.Content class="max-w-sm">
    <Dialog.Header>
      <Dialog.Title>{mn.exams.import.passphraseTitle}</Dialog.Title>
      <Dialog.Description>{mn.exams.import.passphraseHint}</Dialog.Description>
    </Dialog.Header>
    <div class="space-y-1.5 py-2">
      <Label for="import-passphrase">{mn.exams.import.passphraseLabel}</Label>
      <Input
        id="import-passphrase"
        type="password"
        bind:value={passphrase}
        placeholder={mn.exams.import.passphrasePlaceholder}
        onkeydown={(e) => {
          if (e.key === "Enter") unlock();
        }}
      />
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={cancel}>
        {mn.exams.import.cancel}
      </Button>
      <Button onclick={unlock} disabled={!passphrase || opening}>
        {mn.exams.import.unlock}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
