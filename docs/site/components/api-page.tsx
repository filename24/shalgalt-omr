'use client';

import { createOpenAPIPage } from 'fumadocs-openapi/ui';
import 'fumadocs-openapi/css/preset.css';

// The OpenAPI renderer is a client component. The server preloads the spec for each page
// (see app/[lang]/docs/[[...slug]]/page.tsx) and passes the serialized data in as props, so
// no filesystem path is read on the client and the output stays fully static.
export const OpenAPIPage = createOpenAPIPage();
