import { source } from '@/lib/source';
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
} from 'fumadocs-ui/layouts/docs/page';
import { notFound } from 'next/navigation';
import defaultMdxComponents from 'fumadocs-ui/mdx';
import { getMDXComponents } from '@/components/mdx';
import { openapi } from '@/lib/openapi';
import { OpenAPIPage } from '@/components/api-page';

// Content authors write absolute cross-page links like `/dev/architecture` or
// `/user/quick-start` (the dev/ and user/ content trees). The pages actually live under
// `/{locale}/docs/...`, so localize those links here before delegating to Fumadocs' default
// anchor (which prepends the Next.js basePath). External and in-page links pass through.
function localizedAnchor(lang: string) {
  const Anchor = defaultMdxComponents.a as React.FC<React.ComponentProps<'a'>>;
  return function A({ href, ...props }: React.ComponentProps<'a'>) {
    const h = href && /^\/(dev|user)(\/|$)/.test(href) ? `/${lang}/docs${href}` : href;
    return <Anchor href={h} {...props} />;
  };
}

export default async function Page(props: {
  params: Promise<{ lang: string; slug?: string[] }>;
}) {
  const params = await props.params;
  const page = source.getPage(params.slug, params.lang);
  if (!page) notFound();

  const MDX = page.data.body;

  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      <DocsBody>
        <MDX
          components={getMDXComponents({
            a: localizedAnchor(params.lang),
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
