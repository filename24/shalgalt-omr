import type { PageLoad } from "./$types";

// Runtime-resolved ids — not prerenderable. The static adapter's SPA fallback
// serves this client-side route inside the Tauri webview.
export const prerender = false;

export const load: PageLoad = ({ params }) => {
  return {
    examId: Number(params.exam_id),
    variant: params.variant,
  };
};
