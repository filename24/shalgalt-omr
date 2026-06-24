import defaultMdxComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';

// Base MDX component map. The OpenAPI `OpenAPIPage` component is injected per-page in the
// route handler (it needs the server-preloaded spec), so it is not created here.
export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    ...components,
  };
}
