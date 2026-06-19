import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TaskProgress } from "$lib/types/progress";

/**
 * Rule 2 — collect every `task-progress` event the Rust core emits into one store.
 *
 * Built on Svelte 5 runes: components can read `progress.last` / `progress.history`
 * reactively.
 */
class ProgressStore {
  last = $state<TaskProgress | null>(null);
  history = $state<TaskProgress[]>([]);

  #unlisten: UnlistenFn | null = null;

  async start(): Promise<void> {
    if (this.#unlisten) return;
    this.#unlisten = await listen<TaskProgress>("task-progress", (e) => {
      this.last = e.payload;
      this.history = [...this.history.slice(-49), e.payload];
    });
  }

  stop(): void {
    this.#unlisten?.();
    this.#unlisten = null;
  }
}

export const progress = new ProgressStore();

if (typeof window !== "undefined") {
  void progress.start();
}
