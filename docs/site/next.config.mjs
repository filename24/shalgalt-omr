import { createMDX } from 'fumadocs-mdx/next';
import path from 'node:path';

const withMDX = createMDX();

// Content lives in ../dev and ../user (the repo's docs/ tree), OUTSIDE this
// app dir. Point Turbopack's project root at the repo's docs/ directory so it
// is allowed to resolve modules above docs/site/.
const docsRoot = path.resolve(import.meta.dirname, '..');

const isProd = process.env.NODE_ENV === 'production';
// Allow overriding the base path (e.g. local prod testing) without code edits.
const basePath = process.env.DOCS_BASE_PATH ?? (isProd ? '/shalgalt-omr' : '');

/** @type {import('next').NextConfig} */
const config = {
  output: 'export',
  outputFileTracingRoot: docsRoot,
  turbopack: {
    root: docsRoot,
  },
  reactStrictMode: true,
  trailingSlash: true,
  images: { unoptimized: true },
  basePath: basePath || undefined,
  assetPrefix: basePath || undefined,
  env: {
    // Exposed so client code (search link prefixing) can read the base path.
    NEXT_PUBLIC_BASE_PATH: basePath,
  },
};

export default withMDX(config);
