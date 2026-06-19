// `/results/[job_id]` is dynamic — the job id is a runtime row id, so there is
// nothing for adapter-static to crawl at build time. Disable prerendering for
// this route and let the SPA shell (adapter `fallback: "index.html"`) serve any
// concrete `[job_id]`.
export const prerender = false;
