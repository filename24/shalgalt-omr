import { renderLlmsIndex, textResponse } from '@/lib/llms-content';
import { i18n } from '@/lib/i18n';

// Root /llms.txt at the conventional discovery location. Defaults to the
// default language (Mongolian); /en/llms.txt serves the English index.
export const revalidate = false;

export function GET() {
  return textResponse(renderLlmsIndex(i18n.defaultLanguage));
}
