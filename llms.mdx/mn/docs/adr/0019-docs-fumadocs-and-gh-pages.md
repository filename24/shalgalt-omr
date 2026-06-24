# ADR 0019 — Documentation site (https://filename24.github.io/shalgalt-omr/mn/docs/adr/0019-docs-fumadocs-and-gh-pages)



<Callout type="success" title="Accepted · 2026-06-22">
  **Deciders:** filename24 (maintainer) · **Supersedes:** the mdBook/VitePress/redoc tooling
  choice in master plan §12 (updated in the same change set that introduced this ADR).
</Callout>

## Context [#context]

Master plan §12 originally specified two separate documentation toolchains for P7:

* **mdBook** for the developer documentation under `docs/dev/`.
* **VitePress** for the Mongolian user manual under `docs/user/`.
* A third step (P7-03) rendered the `/v1/` OpenAPI document via `redoc`.

Running three tools (mdBook, VitePress, redoc) means three build pipelines, three
themes, three search implementations, and three deploy steps. The OpenAPI page in
particular (`redoc`) sits awkwardly between the two sites. For a single-maintainer
project the operational cost of three toolchains is the dominant concern (master plan
§19 lists "single-maintainer bus factor" as a tracked risk).

## Options considered [#options-considered]

1. **mdBook + VitePress + redoc** (original plan). Three pipelines. mdBook is Rust-native
   (fits the crate docs story) but its theme and search are basic and it has no first-class
   OpenAPI rendering. VitePress covers the user manual well but is a second toolchain.

2. **VitePress for both surfaces.** One tool, but no first-class OpenAPI rendering (would
   still need a redoc/embed step) and weaker API-reference ergonomics.

3. **Fumadocs for everything.** A single Next.js documentation framework that covers the
   developer docs, the Mongolian user manual, and the HTTP API reference in one site:
   * `fumadocs-openapi` generates static MDX pages directly from our `openapi.json`
     (the same document `shalgalt-core` already serves at `/openapi.json`), so P7-03 is a
     first-class part of the site rather than a bolted-on redoc iframe.
   * One theme, one search index (FlexSearch static client — works under `output: export`),
     one deploy step.
   * Two audience trees (`docs/dev`, `docs/user`) modelled as separate sidebar roots, with a
     site-wide i18n locale switcher layered on top (see below).

## Decision [#decision]

<Callout type="info" title="Decision">
  Adopt **Fumadocs** as the single documentation framework for P7 — developer docs, the
  Mongolian user manual, and the HTTP API reference in one site — deployed as a fully static
  build to the existing `gh-pages` branch.
</Callout>

* The Next.js docs app lives in &#x2A;*`docs/site/`** as an isolated package (its own
  `package.json`, not part of the root SvelteKit install) so its Next.js dependency tree
  never mixes with the app's SvelteKit/Tauri tree.
* MDX content lives in &#x2A;*`docs/dev/`*&#x2A; (English) and &#x2A;*`docs/user/`** (Mongolian), honouring
  the directory layout master plan §12 prescribes. `docs/site/` only holds the framework.
* The build is fully static: `next.config` sets `output: "export"`, `trailingSlash: true`,
  `images.unoptimized: true`, and `basePath: "/shalgalt-omr"` (the GitHub Pages project
  subpath). Search runs through `flexsearchStaticClient` + a `staticGET` route so it works
  without a server.
* The HTTP API reference (P7-03) is produced by running
  `cargo run -p shalgalt-core --example dump_openapi -- docs/site/openapi.json` and then
  `fumadocs-openapi`'s `generateFiles` into `docs/dev/api/`. The spec therefore cannot
  drift from the router.

## Internationalization (i18n) [#internationalization-i18n]

The site is multilingual via Fumadocs i18n with two locales:

* **`mn` (Mongolian) — the default locale.** The bare URL
  (`https://filename24.github.io/shalgalt-omr/`) serves Mongolian, matching the product's
  Mongolian-only UI and the "Mongolian-first" priority.
* **`en` (English) — the secondary locale**, reachable via the locale switcher
  (`/shalgalt-omr/en/...`).

Content authoring follows the Fumadocs convention that an &#x2A;*unsuffixed file is the default
language (`mn`)** and a `*.en.mdx` file is the English version; a missing locale falls back
to the default-language file. Two consequences flow from the repository's own language rules:

* **Developer docs (`docs/dev/`) are authored in English** as unsuffixed `.mdx` files. The
  repository rule "code & documentation are English only" forbids a Mongolian translation of
  developer docs, so these pages render in English under **both** locales (the `en` locale
  resolves them directly; the `mn` locale shows the same English content). "English-first when
  developing" is therefore satisfied automatically.
* **User-manual docs (`docs/user/`) are authored in Mongolian** as unsuffixed `.mdx`
  (the `mn` default) with `*.en.mdx` English translations added progressively. Untranslated
  user pages fall back to Mongolian.

Search is locale-aware (`createI18nSearchAPI` on the server side + `flexsearchStaticClient`
with the active locale on the client), and still runs fully in-browser under `output:
export`.

## GitHub Pages coexistence with the updater manifest [#github-pages-coexistence-with-the-updater-manifest]

`gh-pages` already hosts the `tauri-plugin-updater` manifest at
`https://filename24.github.io/shalgalt-omr/latest.json` (ADR 0018, published by
`release.yml`). The P6 publish step already uses `peaceiris/actions-gh-pages@v4` with
`keep_files: true` precisely so documentation can land alongside it.

The docs deploy workflow (`.github/workflows/docs.yml`) mirrors this: it publishes the
static `out/` directory to the **root** of `gh-pages` with `keep_files: true`. Result:

* `https://filename24.github.io/shalgalt-omr/` → documentation home.
* `https://filename24.github.io/shalgalt-omr/latest.json` → updater manifest (untouched).

Because both workflows use `keep_files: true`, neither clobbers the other's output.

## Consequences [#consequences]

* **Positive**: one toolchain, one theme, one search, one deploy; first-class OpenAPI;
  API spec stays in sync with the router; docs and updater coexist on one branch.
* **Negative**: introduces a Next.js/React dependency tree for docs only. Mitigated by
  isolating it in `docs/site/` (separate install) so it never touches the app build.
* **Follow-up**: per-release PDF exports of both manuals (master plan §12) are deferred to
  a later docs iteration; the static HTML site is the v1.0 deliverable.

<Cards>
  <Card href="/adr/0018-optin-updater" title="ADR 0018 — Opt-in updater">
    The `gh-pages` updater manifest (`latest.json`) this docs deploy coexists with via `keep_files`.
  </Card>
</Cards>
