/**
 * Comfortable typography toggle (P2-10).
 *
 * Two modes — "default" (16 px base, 40 px hit targets) and "comfortable"
 * (18 px base, 48 px hit targets). The active mode is mirrored to the
 * `data-comfort` attribute on `<html>` so the CSS selectors in
 * `src/app.css` reflow without per-component rewrites.
 *
 * Persistence uses `localStorage` under `shalgalt-omr.comfort-mode` and
 * mirrors the way `mode-watcher` handles theme persistence so users get a
 * consistent "your preferences stick" experience.
 */

const STORAGE_KEY = "shalgalt-omr.comfort-mode";

export type ComfortMode = "default" | "comfortable";

function readInitial(): ComfortMode {
  if (typeof localStorage === "undefined") return "default";
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "comfortable" ? "comfortable" : "default";
}

function applyToDom(mode: ComfortMode): void {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-comfort", mode);
}

class ComfortModeStore {
  mode = $state<ComfortMode>(readInitial());

  constructor() {
    if (typeof document !== "undefined") {
      applyToDom(this.mode);
    }
  }

  set(mode: ComfortMode): void {
    this.mode = mode;
    applyToDom(mode);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, mode);
    }
  }

  toggle(): void {
    this.set(this.mode === "comfortable" ? "default" : "comfortable");
  }
}

export const comfortMode = new ComfortModeStore();
