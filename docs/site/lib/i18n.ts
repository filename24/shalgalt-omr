import { defineI18n } from 'fumadocs-core/i18n';

// Default = Mongolian (mn). Every locale is prefixed in the URL (`/mn/...`,
// `/en/...`). We intentionally do NOT use `hideLocale: 'default-locale'`: that
// relies on a middleware rewrite to serve the default locale at the root, which
// does not exist under `output: export`. The site root redirects to `/mn/`
// instead (see scripts/write-root-redirect.mjs).
export const i18n = defineI18n({
  defaultLanguage: 'mn',
  languages: ['mn', 'en'],
});
