import { getLLMText } from '@/lib/get-llm-text';
import { source } from '@/lib/source';
import { notFound } from 'next/navigation';

// Per-page raw Markdown endpoint (e.g. /llms.mdx/mn/docs/dev/architecture.md).
// Powers the "Copy Markdown" / "View as Markdown" page actions and lets LLMs
// fetch a single page as plain Markdown. A separate route tree is used because a
// route handler cannot live in the same segment as the page's `page.tsx`, and
// `output: export` does not support rewrites to alias it to `<page>.md`.
//
// The `.md` suffix is load-bearing: under `output: export` a route handler is
// written as a flat file at its pathname, so a section index (`dev`) and its
// children (`dev/architecture`) would collide (file vs directory). Suffixing
// with `.md` makes them `dev.md` (file) and `dev/` (directory), which coexist.
export const revalidate = false;

// Append `.md` to the last slug segment (or use `index.md` for the docs root).
function withMdSuffix(slug?: string[]): string[] {
  if (!slug || slug.length === 0) return ['index.md'];
  return [...slug.slice(0, -1), `${slug[slug.length - 1]}.md`];
}

// Reverse of withMdSuffix: recover the real page slug from the requested path.
function stripMdSuffix(slug?: string[]): string[] | undefined {
  if (!slug || slug.length === 0) return undefined;
  const last = slug[slug.length - 1].replace(/\.md$/, '');
  if (last === 'index') return slug.slice(0, -1);
  return [...slug.slice(0, -1), last];
}

export async function GET(
  _req: Request,
  { params }: { params: Promise<{ lang: string; slug?: string[] }> },
) {
  const { lang, slug } = await params;
  const page = source.getPage(stripMdSuffix(slug), lang);
  if (!page) notFound();

  return new Response(await getLLMText(page), {
    headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
  });
}

export function generateStaticParams() {
  return source
    .generateParams()
    .map(({ lang, slug }) => ({ lang, slug: withMdSuffix(slug) }));
}
