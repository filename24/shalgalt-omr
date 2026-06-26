import { defineDocs, defineConfig } from 'fumadocs-mdx/config';

// Content lives OUTSIDE this app, in the repo's docs/ tree. We point the
// collection at `..` (the repo's docs/ directory) but restrict the file globs
// to the `dev/`, `user/`, and `adr/` subtrees. This excludes the top-level
// docs/*.md files (BLUEPRINT, ARCHITECTURE, ...) and any other stray markdown.
// The three `meta.json` files with `root: true` give us three separate sidebar
// roots ("Developer", "User Manual", and "Architecture Decisions").
//
// `adr/README.md` is the GitHub-facing folder index and is intentionally NOT
// shipped to the site: `adr/index.mdx` is the site section landing page, and the
// glob below excludes the README so it does not become a stray duplicate page.
//
// fumadocs-mdx 15's `defineDocs` accepts a single `dir` string (not an array),
// so we scope with `files` globs instead.
export const docs = defineDocs({
  dir: '..',
  docs: {
    files: [
      'dev/**/*.{md,mdx}',
      'user/**/*.{md,mdx}',
      'adr/**/*.{md,mdx}',
      '!adr/README.md',
    ],
    // Keep the processed Markdown around so the LLM integrations
    // (llms.txt / llms-full.txt / per-page .md routes, see lib/get-llm-text.ts)
    // can emit `page.data.getText('processed')`.
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
  meta: {
    files: [
      'dev/**/*.{json,yaml}',
      'user/**/*.{json,yaml}',
      'adr/**/*.{json,yaml}',
    ],
  },
});

export default defineConfig();
