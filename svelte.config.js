// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // `fallback: "index.html"` produces an SPA shell so dynamic routes
    // (e.g. `/review/[job_id]`) resolve at runtime through the client-side
    // router. The shell is byte-identical to the prerendered home page in
    // this app because `+layout.ts` sets `ssr = false`, so there is no
    // server-rendered content to lose.
    adapter: adapter({ fallback: "index.html" }),
  },
};

export default config;
