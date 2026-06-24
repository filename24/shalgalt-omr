import { docs } from '@/.source/server';
import { loader } from 'fumadocs-core/source';
import { i18n } from '@/lib/i18n';

// Single loader over the combined dev + user content tree.
// `meta.json`/`meta.en.json` with `root: true` give us two sidebar roots
// ("Developer" and "User Manual" / "Хэрэглэгчийн гарын авлага").
export const source = loader({
  i18n,
  baseUrl: '/docs',
  source: docs.toFumadocsSource(),
});
