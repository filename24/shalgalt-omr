import { renderLlmsFull, textResponse } from '@/lib/llms-content';
import { i18n } from '@/lib/i18n';

// Root /llms-full.txt at the conventional discovery location. Defaults to the
// default language (Mongolian); /en/llms-full.txt serves the English corpus.
export const revalidate = false;

export async function GET() {
  return textResponse(await renderLlmsFull(i18n.defaultLanguage));
}
