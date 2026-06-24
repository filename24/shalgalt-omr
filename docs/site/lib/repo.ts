// Repository coordinates and URL helpers shared by the docs-site integrations
// (last-edit lookup, "Edit on GitHub" / "View as Markdown" page actions, and the
// llms.txt / per-page markdown routes).

export const OWNER = 'filename24';
export const REPO = 'shalgalt-omr';

// The docs site is built and published from the `stable` branch
// (see .github/workflows/docs.yml), so source links and last-edit lookups must
// point at `stable`, not the default branch.
export const SHA = 'stable';

// Production origin of the published GitHub Pages site. Combined with the
// Next.js basePath it yields absolute URLs for llms.txt / llms-full.txt, which
// LLMs need in order to fetch the referenced pages.
export const SITE_ORIGIN = 'https://filename24.github.io';

// Next.js basePath (e.g. `/shalgalt-omr` in production, empty in dev). Exposed
// to both server and client via next.config's `env`.
export const BASE_PATH = process.env.NEXT_PUBLIC_BASE_PATH ?? '';

interface PageFile {
  // Absolute path on disk (e.g. /…/docs/user/quick-start.en.mdx).
  absolutePath?: string;
  // Virtualized path relative to the content dir. For i18n it drops the locale
  // suffix, so it is only correct for the default locale — we prefer absolutePath.
  path: string;
}

// Resolve a page's path relative to the repo root (e.g.
// `docs/user/quick-start.en.mdx`), preferring the on-disk absolute path so
// English `*.en.mdx` variants resolve to their real source file.
export function repoRelativePath(page: PageFile): string {
  const abs = page.absolutePath?.replace(/\\/g, '/');
  if (abs) {
    const marker = abs.lastIndexOf('/docs/');
    if (marker !== -1) return abs.slice(marker + 1);
  }
  return `docs/${page.path}`;
}

// GitHub "blob" URL for a page's source file on the published branch.
export function githubBlobUrl(page: PageFile): string {
  return `https://github.com/${OWNER}/${REPO}/blob/${SHA}/${repoRelativePath(page)}`;
}

// In-browser URL (basePath included) of a page's raw markdown, served by the
// `app/llms.mdx/[lang]/docs/[[...slug]]` route. `pageUrl` is the page's own url
// like `/mn/docs/dev/architecture`. The trailing `.md` keeps section-index files
// (e.g. `dev.md`) from colliding with their child directory (`dev/`) under the
// static export's flat file output.
export function markdownUrl(pageUrl: string): string {
  return `${BASE_PATH}/llms.mdx${pageUrl}.md`;
}

// Absolute, publicly fetchable URL for a page (basePath + origin included).
export function absolutePageUrl(pageUrl: string): string {
  return `${SITE_ORIGIN}${BASE_PATH}${pageUrl}`;
}
