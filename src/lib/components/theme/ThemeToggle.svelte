<script lang="ts">
  import { mode, setMode } from "mode-watcher";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import LaptopIcon from "@lucide/svelte/icons/laptop";

  import { mn } from "$lib/i18n";

  // `mode-watcher` exposes the active mode as a runes-friendly accessor —
  // `mode.current` is `"light" | "dark"` (resolved). The user's *choice*
  // (which can include `"system"`) lives in the underlying setter only,
  // so we keep the menu state local rather than reading from mode-watcher.
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button variant="ghost" size="sm" class="w-full justify-start gap-2" {...props}>
        {#if mode.current === "dark"}
          <MoonIcon class="size-4" />
        {:else}
          <SunIcon class="size-4" />
        {/if}
        <span>{mode.current === "dark" ? mn.theme.dark : mn.theme.light}</span>
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-44">
    <DropdownMenu.Item onclick={() => setMode("light")}>
      <SunIcon class="mr-2 size-4" />
      {mn.theme.light}
    </DropdownMenu.Item>
    <DropdownMenu.Item onclick={() => setMode("dark")}>
      <MoonIcon class="mr-2 size-4" />
      {mn.theme.dark}
    </DropdownMenu.Item>
    <DropdownMenu.Item onclick={() => setMode("system")}>
      <LaptopIcon class="mr-2 size-4" />
      {mn.theme.system}
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
