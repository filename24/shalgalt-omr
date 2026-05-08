<script lang="ts">
  import { mn } from "$lib/i18n";
  import QuickActionsWidget from "$lib/components/widgets/QuickActionsWidget.svelte";
  import RecentExamsWidget from "$lib/components/widgets/RecentExamsWidget.svelte";
  import RecentResultsWidget from "$lib/components/widgets/RecentResultsWidget.svelte";
</script>

<section class="container mx-auto space-y-6 p-6 lg:p-8">
  <header class="space-y-1">
    <h2 class="text-2xl font-bold tracking-tight">{mn.dashboard.title}</h2>
    <p class="text-muted-foreground text-sm">{mn.dashboard.subtitle}</p>
  </header>

  <QuickActionsWidget />

  <div class="grid gap-4 lg:grid-cols-2">
    <RecentExamsWidget />
    <RecentResultsWidget />
  </div>
</section>
