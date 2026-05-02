<script lang="ts">
  import { page } from "$app/state";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import PencilRulerIcon from "@lucide/svelte/icons/pencil-ruler";
  import ScanIcon from "@lucide/svelte/icons/scan-line";
  import ClipboardListIcon from "@lucide/svelte/icons/clipboard-list";

  // P0 placeholder labels. Replaced by Mongolian copy from the P1 string-table.
  const items = [
    { href: "/", label: "Dashboard", icon: LayoutDashboardIcon },
    { href: "/editor", label: "Template Editor", icon: PencilRulerIcon },
    { href: "/grade", label: "Grade PDF", icon: ScanIcon },
    { href: "/results", label: "Results", icon: ClipboardListIcon },
  ];
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <div class="px-2 py-1.5 text-sm font-semibold tracking-wide">shalgalt-omr</div>
  </Sidebar.Header>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Workspace</Sidebar.GroupLabel>
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
  <Sidebar.Rail />
</Sidebar.Root>
