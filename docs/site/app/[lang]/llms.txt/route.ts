import { renderLlmsIndex, textResponse } from '@/lib/llms-content';
import { i18n } from '@/lib/i18n';

// Per-locale llms.txt index (e.g. /mn/llms.txt, /en/llms.txt): a compact,
// link-rich Markdown outline of the docs for LLM crawlers. Statically generated
// at build time so it works under `output: export`.
export const revalidate = false;

export async function GET(
  _req: Request,
  { params }: { params: Promise<{ lang: string }> },
) {
  const { lang } = await params;
  return textResponse(renderLlmsIndex(lang));
}

export function generateStaticParams() {
  return i18n.languages.map((lang) => ({ lang }));
}
