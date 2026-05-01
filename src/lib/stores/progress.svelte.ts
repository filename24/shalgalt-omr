import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TaskProgress } from "$lib/types/progress";

/**
 * Rule 2 — Rust가 발행하는 `task-progress` 이벤트를 단일 스토어에 모은다.
 *
 * Svelte 5 runes 기반: `progress.last`/`progress.history`를 컴포넌트가 reactive하게 읽을 수 있다.
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
