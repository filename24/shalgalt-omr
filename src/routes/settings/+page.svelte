<script lang="ts">
  import { toast } from "svelte-sonner";
  import { mn } from "$lib/i18n";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { comfortMode } from "$lib/stores/comfortMode.svelte";
  import { updaterPreference } from "$lib/stores/updaterPreference.svelte";
  import RefreshIcon from "@lucide/svelte/icons/refresh-cw";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import TextIcon from "@lucide/svelte/icons/case-sensitive";

  // Run a manual update check. The store guards on the opt-in flag, so this is
  // a no-op (no network) when auto-update is off — the button is only shown
  // while the preference is enabled, but the guard is the real invariant.
  async function checkNow(): Promise<void> {
    toast.loading(mn.updater.checking, { id: "updater-check" });
    try {
      const update = await updaterPreference.checkForUpdate();
      if (update) {
        toast.success(mn.updater.available, {
          id: "updater-check",
          description: update.version,
        });
      } else {
        toast.success(mn.updater.upToDate, { id: "updater-check" });
      }
    } catch (e) {
      toast.error(mn.updater.checkFailed, {
        id: "updater-check",
        description: String(e),
      });
    }
  }
</script>

<section class="container mx-auto max-w-3xl space-y-6 p-6 lg:p-8">
  <header class="space-y-1">
    <h2 class="text-2xl font-bold tracking-tight">{mn.settings.title}</h2>
    <p class="text-muted-foreground text-sm">{mn.settings.subtitle}</p>
  </header>

  <Card.Root>
    <Card.Header>
      <Card.Title>{mn.settings.appearance}</Card.Title>
    </Card.Header>
    <Card.Content>
      <Button
        variant={comfortMode.mode === "comfortable" ? "secondary" : "ghost"}
        class="w-full justify-start gap-2"
        onclick={() => comfortMode.toggle()}
        title={mn.theme.comfortHint}
      >
        <TextIcon class="size-4" />
        <span>
          {comfortMode.mode === "comfortable"
            ? mn.theme.comfortOn
            : mn.theme.comfortOff}
        </span>
      </Button>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>{mn.settings.updates}</Card.Title>
    </Card.Header>
    <Card.Content class="space-y-2">
      <Button
        variant={updaterPreference.enabled ? "secondary" : "ghost"}
        class="w-full justify-start gap-2"
        onclick={() => updaterPreference.set(!updaterPreference.enabled)}
        title={mn.theme.autoUpdateHint}
      >
        <DownloadIcon class="size-4" />
        <span>
          {updaterPreference.enabled
            ? mn.theme.autoUpdateOn
            : mn.theme.autoUpdateOff}
        </span>
      </Button>

      {#if updaterPreference.enabled}
        <Button
          variant="outline"
          class="w-full justify-start gap-2"
          disabled={updaterPreference.checking}
          onclick={checkNow}
        >
          <RefreshIcon class="size-4" />
          <span>{mn.settings.checkNow}</span>
        </Button>
      {/if}
    </Card.Content>
  </Card.Root>
</section>
