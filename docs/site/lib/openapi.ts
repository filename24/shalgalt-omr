import { createOpenAPI } from 'fumadocs-openapi/server';
import path from 'node:path';

// Local committed spec — never fetched over the network.
export const openapi = createOpenAPI({
  input: [path.resolve(process.cwd(), 'openapi.json')],
});
