// `/review/[job_id]` is dynamic — the job id is generated at runtime by
// `/grade`, so there is nothing for adapter-static to crawl at build time.
// Disable prerendering for this route only and let the SPA shell (configured
// via `fallback: "index.html"` on the adapter) serve any concrete `[job_id]`.
export const prerender = false;
