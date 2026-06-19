/**
 * Auto-update opt-in preference (P6 distribution).
 *
 * Master plan §20 hard gate: the auto-updater is OFF by default so offline
 * schools never see an update prompt. The toggle persists exactly like the
 * comfort-mode store — `localStorage` under `shalgalt-omr.auto-update` — so the
 * choice survives restarts.
 *
 * The store owns the single place that calls the updater's `check()`. The
 * runtime invariant is strict: when the preference is `false`, `check()` is
 * never invoked, so no network access happens and no prompt can appear.
 */
import { check, type Update } from "@tauri-apps/plugin-updater";

const STORAGE_KEY = "shalgalt-omr.auto-update";

function readInitial(): boolean {
  if (typeof localStorage === "undefined") return false;
  // Default OFF: only the explicit string "true" opts in.
  return localStorage.getItem(STORAGE_KEY) === "true";
}

class UpdaterPreferenceStore {
  /** Whether the user has opted into automatic update checks. Default false. */
  enabled = $state<boolean>(readInitial());

  /** True while a `check()` round-trip is in flight, to disable re-entry. */
  checking = $state<boolean>(false);

  set(enabled: boolean): void {
    this.enabled = enabled;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, enabled ? "true" : "false");
    }
  }

  toggle(): void {
    this.set(!this.enabled);
  }

  /**
   * Check for an available update — but ONLY when the user has opted in.
   * Returns the pending `Update` when one exists, `null` when up to date, and
   * `null` (without ever touching the network) when the preference is off.
   *
   * This is the one and only call site for the updater's `check()`. Callers
   * outside an opt-in flow must not invoke `check()` directly.
   */
  async checkForUpdate(): Promise<Update | null> {
    if (!this.enabled) return null;
    if (this.checking) return null;
    this.checking = true;
    try {
      return await check();
    } finally {
      this.checking = false;
    }
  }
}

export const updaterPreference = new UpdaterPreferenceStore();
