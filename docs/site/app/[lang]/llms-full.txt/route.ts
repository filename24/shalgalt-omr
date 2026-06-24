import { renderLlmsFull, textResponse } from '@/lib/llms-content';
import { i18n } from '@/lib/i18n';

// Per-locale llms-full.txt (e.g. /mn/llms-full.txt): the full processed Markdown
// of every page in that locale concatenated into one document for LLM ingestion.
// Statically generated at build time so it works under `output: export`.
export const revalidate = false;

export async function GET(
  _req: Request,
  { params }: { params: Promise<{ lang: string }> },
) {
  const { lang } = await params;
  return textResponse(await renderLlmsFull(lang));
}

export function generateStaticParams() {
  return i18n.languages.map((lang) => ({ lang }));
}
