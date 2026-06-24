// Prebuild step:
//  1. Generate static MDX API-reference pages from the local OpenAPI spec
//     into ../dev/api (the dev sidebar references `api` as a folder).
//  2. Copy downloadable .shalgalt samples into public/samples (best effort).
//
// The spec is read from the committed local file; nothing is fetched.
import { generateFiles } from 'fumadocs-openapi';
import { createOpenAPI } from 'fumadocs-openapi/server';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const siteRoot = path.resolve(here, '..');
const specPath = path.join(siteRoot, 'openapi.json');
const apiOutDir = path.resolve(siteRoot, '../dev/api');

async function generateApiDocs() {
  if (!fs.existsSync(specPath)) {
    console.warn(`[openapi] spec not found at ${specPath}; skipping API doc generation`);
    return;
  }

  fs.mkdirSync(apiOutDir, { recursive: true });

  const openapi = createOpenAPI({ input: [specPath] });

  await generateFiles({
    input: openapi,
    output: apiOutDir,
    // One MDX file per operation, grouped by tag, with the APIPage component.
    per: 'operation',
    groupBy: 'tag',
  });

  // Ensure ordering meta exists for the generated folder.
  const metaPath = path.join(apiOutDir, 'meta.json');
  if (!fs.existsSync(metaPath)) {
    fs.writeFileSync(
      metaPath,
      JSON.stringify(
        {
          title: 'HTTP API',
          description: 'REST API reference generated from the OpenAPI spec.',
          pages: ['...'],
        },
        null,
        2,
      ) + '\n',
    );
  }

  console.log(`[openapi] generated API docs into ${apiOutDir}`);
}

function copySamples() {
  const samplesSrc = path.resolve(siteRoot, '../user/samples');
  const samplesDest = path.join(siteRoot, 'public/samples');
  if (!fs.existsSync(samplesSrc)) {
    console.warn('[samples] no samples directory; skipping copy');
    return;
  }
  fs.mkdirSync(samplesDest, { recursive: true });
  const files = fs.readdirSync(samplesSrc).filter((f) => f.endsWith('.shalgalt'));
  for (const f of files) {
    fs.copyFileSync(path.join(samplesSrc, f), path.join(samplesDest, f));
  }
  console.log(`[samples] copied ${files.length} .shalgalt file(s) into public/samples`);
}

async function main() {
  await generateApiDocs();
  copySamples();
}

main().catch((err) => {
  console.error('[prebuild] failed:', err);
  process.exit(1);
});
