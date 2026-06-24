import { source } from '@/lib/source';
import { absolutePageUrl } from '@/lib/repo';

// Render a single page as LLM-friendly Markdown: a title + absolute source URL
// header followed by the processed Markdown body. Used by the llms-full.txt and
// per-page `.md` routes.
export async function getLLMText(
  page: (typeof source)['$inferPage'],
): Promise<string> {
  const processed = await page.data.getText('processed');

  return `# ${page.data.title} (${absolutePageUrl(page.url)})

${processed}`;
}
