import { source } from '@/lib/source';
import { llms } from 'fumadocs-core/source';
import { getLLMText } from '@/lib/get-llm-text';
import { SITE_ORIGIN, BASE_PATH } from '@/lib/repo';

// The llms index emits root-relative links (`/mn/docs/...`). Rewrite them to
// absolute URLs (origin + basePath) so an LLM reading the published llms.txt can
// actually fetch the referenced pages.
function absolutize(markdown: string): string {
  return markdown.replaceAll('](/', `](${SITE_ORIGIN}${BASE_PATH}/`);
}

// Compact, link-rich Markdown outline of the docs for a locale (llms.txt).
export function renderLlmsIndex(lang: string): string {
  return absolutize(llms(source).index(lang));
}

// Full processed Markdown of every page in a locale concatenated (llms-full.txt).
export async function renderLlmsFull(lang: string): Promise<string> {
  const pages = source.getPages(lang);
  const scanned = await Promise.all(pages.map(getLLMText));
  return scanned.join('\n\n');
}

export const textResponse = (body: string) =>
  new Response(body, {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
