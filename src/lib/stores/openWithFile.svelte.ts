import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

/**
 * P4-07 — route OS "open with" / file-association launches into the import flow.
 *
 * Two delivery paths converge here:
 *  - **Warm:** the app is already running and the OS forwards a double-clicked
 *    `.shalgalt` path on the `open-with-file` event (macOS `RunEvent::Opened`,
 *    or the single-instance callback on Windows/Linux).
 *  - **Cold:** the app was launched by opening the file. The path was captured
 *    from argv before the webview attached its listener and is drained once via
 *    the `take_pending_open_file` command.
 *
 * Both navigate to `/exams/import?path=…`, which performs the actual
 * `project_open` round-trip. Started from the root layout's `onMount` so the
 * SvelteKit router is ready before any `goto`.
 */
function importUrl(path: string): string {
  return `/exams/import?path=${encodeURIComponent(path)}`;
}

class OpenWithFileStore {
  #unlisten: UnlistenFn | null = null;

  async start(): Promise<void> {
    if (this.#unlisten) return;

    this.#unlisten = await listen<string>("open-with-file", (e) => {
      if (e.payload) void goto(importUrl(e.payload));
    });

    // Drain a cold-start path, if any. In a plain browser dev context the
    // command is absent, so swallow the invoke error.
    try {
      const pending = await invoke<string | null>("take_pending_open_file");
      if (pending) void goto(importUrl(pending));
    } catch {
      // Not running inside Tauri — nothing to drain.
    }
  }

  stop(): void {
    this.#unlisten?.();
    this.#unlisten = null;
  }
}

export const openWithFile = new OpenWithFileStore();
