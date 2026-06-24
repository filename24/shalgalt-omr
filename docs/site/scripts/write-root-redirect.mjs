// Post-build: write out/index.html so the GitHub Pages project root
// (https://filename24.github.io/shalgalt-omr/) redirects to the default locale.
//
// The default locale (mn) is served at /{basePath}/mn/. A relative redirect (`./mn/`)
// resolves correctly regardless of the configured basePath, so nothing is hard-coded here.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.resolve(here, '..', 'out');
const target = './mn/';

const html = `<!doctype html>
<html lang="mn">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta http-equiv="refresh" content="0; url=${target}" />
    <link rel="canonical" href="${target}" />
    <title>shalgalt-omr</title>
  </head>
  <body>
    Redirecting to the <a href="${target}">documentation</a>…
  </body>
</html>
`;

if (!fs.existsSync(outDir)) {
  console.warn(`[redirect] ${outDir} does not exist; did next build run? skipping`);
  process.exit(0);
}

fs.writeFileSync(path.join(outDir, 'index.html'), html);
console.log('[redirect] wrote out/index.html -> ./mn/');
