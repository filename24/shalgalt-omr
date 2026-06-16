import type { PageLoad } from "./$types";

// `/exams/[exam_id]` is dynamic — the exam id is a runtime database key, so
// there is nothing for adapter-static to crawl at build time. Disable
// prerendering for this route only and let the SPA shell (configured via
// `fallback: "index.html"` on the adapter) serve any concrete `[exam_id]`.
export const prerender = false;

export const load: PageLoad = ({ params }) => {
  return { examId: Number(params.exam_id) };
};
