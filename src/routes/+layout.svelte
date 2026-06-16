<script lang="ts">
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Separator } from "$lib/components/ui/separator";
  import { Toaster } from "$lib/components/ui/sonner";
  import AppSidebar from "$lib/components/shell/AppSidebar.svelte";
  import StatusBar from "$lib/components/shell/StatusBar.svelte";
  import { page } from "$app/state";
  import { mn } from "$lib/i18n";
  // Eagerly hydrate the comfort store so its constructor mirrors the
  // persisted choice onto `<html data-comfort="…">` before first paint.
  import "$lib/stores/comfortMode.svelte";

  let { children } = $props();

  const titleByPath: Record<string, string> = {
    "/": mn.nav.dashboard,
    "/editor": mn.nav.editor,
    "/grade": mn.nav.grade,
    "/exams": mn.nav.exams,
    "/results": mn.nav.results,
  };
  // `/review/[job_id]` and `/exams/[exam_id]` are dynamic — match the prefix
  // instead of the exact path.
  const pageTitle = $derived(
    titleByPath[page.url.pathname] ??
      (page.url.pathname.startsWith("/review/")
        ? mn.nav.review
        : page.url.pathname.startsWith("/exams/")
          ? mn.nav.exams
          : ""),
  );
</script>

<ModeWatcher defaultMode="light" />

<Sidebar.Provider>
  <AppSidebar />
  <Sidebar.Inset>
    <header
      class="bg-background sticky top-0 z-10 flex h-12 shrink-0 items-center gap-2 border-b px-3"
    >
      <Sidebar.Trigger class="-ml-1" />
      <Separator orientation="vertical" class="mr-2 h-4" />
      <h1 class="text-sm font-medium">{pageTitle}</h1>
    </header>
    <main class="flex-1 overflow-auto">
      {@render children?.()}
    </main>
    <StatusBar />
  </Sidebar.Inset>
</Sidebar.Provider>

<Toaster richColors position="bottom-right" />
