import defaultMdxComponents from 'fumadocs-ui/mdx';
import { Tab, Tabs } from 'fumadocs-ui/components/tabs';
import { Step, Steps } from 'fumadocs-ui/components/steps';
import { Accordion, Accordions } from 'fumadocs-ui/components/accordion';
import { TypeTable } from 'fumadocs-ui/components/type-table';
import { File, Files, Folder } from 'fumadocs-ui/components/files';
import type { MDXComponents } from 'mdx/types';

// Base MDX component map. We register the full set of Fumadocs UI building blocks
// (Callout/Card come from `defaultMdxComponents`; Tabs/Steps/Accordions/TypeTable/Files
// are added here) so content authors can use them in `.mdx` files without per-file
// imports. The OpenAPI `OpenAPIPage` component is injected per-page in the route handler
// (it needs the server-preloaded spec), so it is not created here.
export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    Tab,
    Tabs,
    Step,
    Steps,
    Accordion,
    Accordions,
    TypeTable,
    File,
    Files,
    Folder,
    ...components,
  };
}
