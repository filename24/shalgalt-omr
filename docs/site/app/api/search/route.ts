import { source } from '@/lib/source';
import { createFromSource } from 'fumadocs-core/search/server';

// Statically cached so it works in a fully static export (search runs in-browser).
export const revalidate = false;

export const { staticGET: GET } = createFromSource(source, {
  // Locale-aware index; required for our mn/en i18n.
  localeMap: {
    mn: { language: undefined },
    en: { language: 'english' },
  },
});
