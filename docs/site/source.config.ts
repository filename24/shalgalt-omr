import { defineDocs, defineConfig } from 'fumadocs-mdx/config';

// Content lives OUTSIDE this app, in the repo's docs/ tree. We point the
// collection at `..` (the repo's docs/ directory) but restrict the file globs
// to ONLY the `dev/` and `user/` subtrees. This excludes docs/adr/, the
// top-level docs/*.md files (BLUEPRINT, ARCHITECTURE, ...), and any other
// stray markdown. The two `meta.json` files with `root: true` give us two
// separate sidebar roots ("Developer" and "User Manual").
//
// fumadocs-mdx 15's `defineDocs` accepts a single `dir` string (not an array),
// so we scope with `files` globs instead.
export const docs = defineDocs({
  dir: '..',
  docs: {
    files: ['dev/**/*.{md,mdx}', 'user/**/*.{md,mdx}'],
  },
  meta: {
    files: ['dev/**/*.{json,yaml}', 'user/**/*.{json,yaml}'],
  },
});

export default defineConfig();
