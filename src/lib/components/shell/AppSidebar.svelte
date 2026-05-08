<script lang="ts">
  import { page } from "$app/state";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import PencilRulerIcon from "@lucide/svelte/icons/pencil-ruler";
  import ScanIcon from "@lucide/svelte/icons/scan-line";
  import ClipboardListIcon from "@lucide/svelte/icons/clipboard-list";
  import { mn } from "$lib/i18n";
  import ThemeToggle from "$lib/components/theme/ThemeToggle.svelte";
  import ComfortToggle from "$lib/components/theme/ComfortToggle.svelte";

  const items = [
    { href: "/", label: mn.nav.dashboard, icon: LayoutDashboardIcon },
    { href: "/editor", label: mn.nav.editor, icon: PencilRulerIcon },
    { href: "/grade", label: mn.nav.grade, icon: ScanIcon },
    { href: "/results", label: mn.nav.results, icon: ClipboardListIcon },
  ];
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <div class="px-2 py-1.5 text-sm font-semibold tracking-wide">{mn.app.title}</div>
  </Sidebar.Header>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>{mn.nav.group}</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each items as item (item.href)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={page.url.pathname === item.href}>
                {#snippet child({ props })}
                  <a href={item.href} {...props}>
                    <item.icon />
                    <span>{item.label}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
  <Sidebar.Footer class="border-sidebar-border border-t">
    <div class="space-y-1 p-1">
      <ComfortToggle />
      <ThemeToggle />
    </div>
  </Sidebar.Footer>
  <Sidebar.Rail />
</Sidebar.Root>
