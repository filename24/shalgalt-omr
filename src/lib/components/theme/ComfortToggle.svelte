<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import TextIcon from "@lucide/svelte/icons/case-sensitive";

  import { mn } from "$lib/i18n";
  import { comfortMode } from "$lib/stores/comfortMode.svelte";
</script>

<Button
  variant={comfortMode.mode === "comfortable" ? "secondary" : "ghost"}
  size="sm"
  class="w-full justify-start gap-2"
  onclick={() => comfortMode.toggle()}
  title={mn.theme.comfortHint}
>
  <TextIcon class="size-4" />
  <span>
    {comfortMode.mode === "comfortable" ? mn.theme.comfortOn : mn.theme.comfortOff}
  </span>
</Button>
