# Testing (https://filename24.github.io/shalgalt-omr/en/docs/dev/testing)



## Rust [#rust]

Unit tests live in `#[cfg(test)]` modules next to the code; integration tests live in each
crate's `tests/` directory (one binary per file). Run everything from the workspace root:

```bash title="Run the Rust suites"
cargo test --workspace            # unit + integration + ts-rs export
cargo test -p shalgalt-core --doc # doc tests
```

Notable suites:

<Accordions>
  <Accordion title="shalgalt-core">
    `template_serde` round-trips an `OmrTemplate` against a golden fixture
    (`tests/fixtures/template-v1.json`), catching accidental Rule 3 schema breakage. The
    grading engine has unit coverage for scoring, partial answer keys, and confidence
    banding. The `/v1/` router is exercised in-process over a `MemoryStore` without binding a
    socket.
  </Accordion>

  <Accordion title="shalgalt-fileformat">
    `roundtrip` (write → read, encrypted and plaintext, streamed 1 MiB entry, deterministic
    order, manifest-has-no-PII), `bad_passphrase`, and `forward_compat` (a `format_version`
    newer than supported is refused before any decryption).
  </Accordion>

  <Accordion title="shalgalt-cv">
    Fixture-driven detection tests under `tests/fixtures/`.
  </Accordion>
</Accordions>

## Frontend [#frontend]

```bash title="Run the frontend suites"
pnpm test          # vitest run
pnpm check         # svelte-kit sync + svelte-check (also catches Rust↔TS type drift)
```

Unit tests cover utilities, data transforms, and stores. Type generation drift (Rust struct
vs generated TS) surfaces as a `pnpm check` failure.

## Generated artifacts used by docs [#generated-artifacts-used-by-docs]

Two examples produce artifacts the documentation consumes — they are also a lightweight
smoke test that the public APIs still compile:

```bash title="Regenerate documentation artifacts"
# OpenAPI document feeding the HTTP API reference (P7-03)
cargo run -p shalgalt-core --example dump_openapi -- docs/site/openapi.json

# The three sample .shalgalt projects shipped with the user manual (P7-04)
cargo run -p shalgalt-fileformat --example generate_samples
```

## Coverage target [#coverage-target]

<Callout type="info" title="80%+ on business logic">
  The repo targets &#x2A;*80%+** coverage on business logic. Prefer fixing the implementation over
  weakening a test; only change a test when the test itself is wrong.
</Callout>
