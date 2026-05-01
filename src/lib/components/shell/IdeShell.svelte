<!--
  P0 placeholder. P1에서 paneforge 기반 IDE 셸로 교체된다 (좌:트리 / 중:캔버스 / 우:인스펙터).
-->
<script lang="ts">
  let { children } = $props();
</script>

<div class="flex h-full w-full">
  {@render children?.()}
</div>
