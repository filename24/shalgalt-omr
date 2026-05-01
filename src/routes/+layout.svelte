<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";

  let { children } = $props();

  const nav = [
    { href: "/", label: "Dashboard" },
    { href: "/editor", label: "Template Editor" },
    { href: "/grade", label: "Grade PDF" },
    { href: "/results", label: "Results" },
  ];
</script>

<div class="flex h-screen w-screen overflow-hidden">
  <aside
    class="w-56 shrink-0 border-r border-[var(--color-border)] bg-[var(--color-surface)] p-3"
  >
    <h1 class="mb-4 text-sm font-semibold tracking-wide text-[var(--color-text-muted)]">
      shalgalt-omr
    </h1>
    <nav class="flex flex-col gap-1">
      {#each nav as item (item.href)}
        <a
          href={item.href}
          class="rounded-md px-3 py-2 text-sm transition hover:bg-[var(--color-surface-elevated)]"
          class:bg-[var(--color-surface-elevated)]={page.url.pathname === item.href}
        >
          {item.label}
        </a>
      {/each}
    </nav>
  </aside>
  <main class="flex-1 overflow-auto">
    {@render children?.()}
  </main>
</div>
