<script lang="ts">
  import { progress } from "$lib/stores/progress.svelte";
  import { gradePdf } from "$lib/ipc/scan";

  let pdfPath = $state("");
  let templateId = $state(1);
  let job = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function start() {
    error = null;
    try {
      const r = await gradePdf(pdfPath, templateId);
      job = r.task_id;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="p-8">
  <h2 class="mb-2 text-2xl font-bold">Grade PDF</h2>
  <p class="mb-6 text-sm text-[var(--color-text-muted)]">
    Rule 1: 파일을 첨부하지 말고 <strong>로컬 절대 경로</strong>만 전달합니다.
  </p>

  <label class="block text-sm">
    PDF 절대 경로
    <input
      bind:value={pdfPath}
      class="mt-1 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm"
      placeholder="/Users/.../scans/2026-05-01.pdf"
    />
  </label>
  <label class="mt-3 block text-sm">
    Template ID
    <input
      type="number"
      bind:value={templateId}
      class="mt-1 w-32 rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm"
    />
  </label>

  <button
    type="button"
    onclick={start}
    class="mt-4 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-black"
  >
    채점 시작
  </button>

  {#if error}
    <p class="mt-4 text-sm text-red-400">{error}</p>
  {/if}
  {#if job}
    <p class="mt-4 text-sm">task_id: <code>{job}</code></p>
  {/if}

  <div class="mt-6 rounded-md border border-[var(--color-border)] p-4">
    <p class="text-xs uppercase text-[var(--color-text-muted)]">Progress</p>
    <p class="mt-1 text-sm">
      stage = {progress.last?.stage ?? "—"} ({progress.last?.processed ?? 0} /
      {progress.last?.total ?? 0})
    </p>
  </div>
</section>
