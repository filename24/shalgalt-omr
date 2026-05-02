<!--
  P0 placeholder. Replaced in P1 by the paneforge-driven IDE shell
  (left: tree / center: canvas / right: inspector).
-->
<script lang="ts">
  let { children } = $props();
</script>

<div class="flex h-full w-full">
  {@render children?.()}
</div>
