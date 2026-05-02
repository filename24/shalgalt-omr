import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

/**
 * Vitest config kept separate from `vite.config.js` because the latter loads
 * the full SvelteKit plugin (which expects a Tauri-aware dev server). The
 * round-trip parity tests in `src/lib/types/template.test.ts` are pure-TS and
 * only need plain Svelte resolution + the `$lib` alias.
 */
export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, "src/lib"),
    },
    conditions: ["browser"],
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
