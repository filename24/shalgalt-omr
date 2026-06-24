# Contributing (https://filename24.github.io/shalgalt-omr/mn/docs/dev/contributing)



# Contributing [#contributing]

## Branching model [#branching-model]

* **`stable`** — release branch. CI publishes from here. Never commit directly.
* **`develop`** — integration branch. Feature PRs land here.
* **`feat/<phase>-<topic>`** — work branches off `develop`. One feature per PR.

`session/*` branches are local isolation only and must never be pushed to the remote.

## Commit conventions [#commit-conventions]

Conventional Commits, one short imperative subject line:

```text
<type>: <description>

<optional body explaining the *why* if the diff doesn't>
```

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `ci`.

## Language rules (mandatory) [#language-rules-mandatory]

Two languages, no overlap:

* **Code and documentation are English** — every comment, doc comment, identifier, log
  message, error string, commit message, PR description, and Markdown file in the repo.
* **End-user UI text is Mongolian** (Cyrillic) — page titles, buttons, labels, validation
  and toast messages, help text. UI strings come from the single string table at
  `src/lib/i18n/`; components never hard-code copy.

`AppError.code` stays English (it is a stable identifier); the frontend maps the code to a
Mongolian message in the string table. A reviewer should treat any non-English string in
source or docs as a blocker, and any hard-coded user-facing string outside the i18n table
as a blocker too.

## The four hard rules [#the-four-hard-rules]

A change is rejected if it violates any of [the four architecture rules](/dev/architecture):
path-strings-only IPC, no UI freeze on batch work, single-JSON template format, and the
host-owned axum lifecycle. When in doubt, read the relevant crate's `AGENTS.md`.

## ADRs [#adrs]

Architectural or operational decisions that are hard to reverse get an ADR under
`docs/adr/` (`NNNN-kebab-title.md`, no gaps). Re-litigating a locked decision requires a new
ADR **plus** a master-plan update in the same PR. See the [ADR index](/dev/adr).

## PR checklist [#pr-checklist]

Before requesting review:

* [ ] `cargo fmt --all -- --check` is clean.
* [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` is clean.
* [ ] `cargo test --workspace` passes (unit + integration + ts-rs export).
* [ ] `pnpm check` passes (no Rust↔TS type drift).
* [ ] New behaviour has tests; coverage target is 80%+.
* [ ] No non-English strings in code/docs; no hard-coded UI copy outside the i18n table.
* [ ] No secrets committed; the axum server still binds `127.0.0.1` by default.
* [ ] Branch is rebased on `develop` and conflicts are resolved.

## Continuous integration [#continuous-integration]

CI runs on every push to `develop`/`stable` and on PRs targeting them:

| Job          | Purpose                                                                |
| ------------ | ---------------------------------------------------------------------- |
| `web-check`  | `pnpm install --frozen-lockfile`, `svelte-kit sync`, `pnpm check`.     |
| `rust-check` | apt deps → `cargo fmt --check` → `clippy -D warnings` → `cargo check`. |

Both jobs must be green before merge. Release bundling (signed installers, pdfium download,
packaging) and the updater manifest are handled by `release.yml` on `v*` tags. The
documentation site is built and published to GitHub Pages by `docs.yml`.
