import { source } from '@/lib/source';
import { DocsBody, DocsDescription, DocsPage, DocsTitle } from 'fumadocs-ui/page';
import {
  MarkdownCopyButton,
  ViewOptionsPopover,
} from 'fumadocs-ui/layouts/docs/page';
import { notFound } from 'next/navigation';
import defaultMdxComponents from 'fumadocs-ui/mdx';
import { getMDXComponents } from '@/components/mdx';
import { openapi } from '@/lib/openapi';
import { OpenAPIPage } from '@/components/api-page';
import { getLastEdit } from '@/lib/last-edit';
import { githubBlobUrl, markdownUrl } from '@/lib/repo';
import { Card } from 'fumadocs-ui/components/card';

// Content authors write absolute cross-page links like `/dev/architecture`,
// `/user/quick-start`, or `/adr/0009-aruco-markers` (the dev/, user/, and adr/ content
// trees). The pages actually live under `/{locale}/docs/...`. Rewrite internal links into
// the localized route before delegating to the underlying component.
const INTERNAL_LINK = /^\/(dev|user|adr)(\/|$)/;
const localizeHref = (lang: string, href?: string) =>
  href && INTERNAL_LINK.test(href) ? `/${lang}/docs${href}` : href;

// Anchor used for plain Markdown links. External and in-page links pass through Fumadocs'
// default anchor (which prepends the Next.js basePath).
function localizedAnchor(lang: string) {
  const Anchor = defaultMdxComponents.a as React.FC<React.ComponentProps<'a'>>;
  return function A({ href, ...props }: React.ComponentProps<'a'>) {
    return <Anchor href={localizeHref(lang, href)} {...props} />;
  };
}

// `<Card href="/dev/..." />` renders its own anchor (not the Markdown `a`), so it needs the
// same locale rewriting to keep cross-tree navigation cards working under `/{locale}/docs`.
function localizedCard(lang: string) {
  return function LocalizedCard({ href, ...props }: React.ComponentProps<typeof Card>) {
    return <Card href={localizeHref(lang, href)} {...props} />;
  };
}

export default async function Page(props: {
  params: Promise<{ lang: string; slug?: string[] }>;
}) {
  const params = await props.params;
  const page = source.getPage(params.slug, params.lang);
  if (!page) notFound();

  const MDX = page.data.body;
  const lastUpdate = await getLastEdit(page);
  const pageMarkdownUrl = markdownUrl(page.url);

  return (
    <DocsPage
      toc={page.data.toc}
      full={page.data.full}
      lastUpdate={lastUpdate ?? undefined}
    >
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      {/* LLM page actions: copy/view the raw Markdown, open in GitHub / an AI chat. */}
      <div className="flex flex-row items-center gap-2 border-b border-fd-border pb-6">
        <MarkdownCopyButton markdownUrl={pageMarkdownUrl} />
        <ViewOptionsPopover
          markdownUrl={pageMarkdownUrl}
          githubUrl={githubBlobUrl(page)}
        />
      </div>
      <DocsBody>
        <MDX
          components={getMDXComponents({
            a: localizedAnchor(params.lang),
            Card: localizedCard(params.lang),
            // Preload the spec on the server so the client renderer gets serialized data
            // (no filesystem path read on the client; output stays static).
            OpenAPIPage: async (props) => (
              <OpenAPIPage {...(await openapi.preloadOpenAPIPage(page))} {...props} />
            ),
          })}
        />
      </DocsBody>
    </DocsPage>
  );
}

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: {
  params: Promise<{ lang: string; slug?: string[] }>;
}) {
  const params = await props.params;
  const page = source.getPage(params.slug, params.lang);
  if (!page) notFound();
  return {
    title: page.data.title,
    description: page.data.description,
  };
}
