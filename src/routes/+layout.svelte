<script lang="ts">
  import "../app.css";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Separator } from "$lib/components/ui/separator";
  import { Toaster } from "$lib/components/ui/sonner";
  import AppSidebar from "$lib/components/shell/AppSidebar.svelte";
  import { page } from "$app/state";

  let { children } = $props();

  // P0 placeholder labels. Replaced by Mongolian copy from the P1 string-table.
  const titleByPath: Record<string, string> = {
    "/": "Dashboard",
    "/editor": "Template Editor",
    "/grade": "Grade PDF",
    "/results": "Results",
  };
  const pageTitle = $derived(titleByPath[page.url.pathname] ?? "");
</script>

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
  </Sidebar.Inset>
</Sidebar.Provider>

<Toaster richColors position="bottom-right" />
