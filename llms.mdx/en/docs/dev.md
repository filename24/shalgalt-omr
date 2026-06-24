# Overview (https://filename24.github.io/shalgalt-omr/en/docs/dev)



shalgalt-omr is a **local-first OMR (Optical Mark Recognition) grading IDE**: a Tauri 2
desktop application with a SvelteKit frontend and a Rust core that grades scanned OMR PDFs
fully offline, persists results in SQLite, and exposes an optional background HTTP API for
integrations.

This site has two audiences:

<Cards>
  <Card href="/dev/architecture" title="Developer documentation (English)">
    Architecture, the Cargo workspace map, how to build and run on each OS, the contribution
    workflow, the ADR index, and the HTTP API reference.
  </Card>

  <Card href="/user" title="Хэрэглэгчийн гарын авлага (Mongolian)">
    Installation and day-to-day use of the application.
  </Card>
</Cards>

## What it does [#what-it-does]

<Steps>
  <Step>
    ### Design [#design]

    Lay out an OMR answer sheet in a visual editor (ArUco corner markers + bubble groups).
  </Step>

  <Step>
    ### Generate [#generate]

    Produce a printable PDF of that sheet.
  </Step>

  <Step>
    ### Grade [#grade]

    Process stacks of scanned/photographed sheets offline using an OpenCV pipeline
    (adaptive threshold, auto-deskew, per-bubble confidence scoring).
  </Step>

  <Step>
    ### Review [#review]

    Inspect low-confidence sheets bubble-by-bubble.
  </Step>

  <Step>
    ### Export [#export]

    Write results to Excel, or share a whole project as an encrypted `.shalgalt` file.
  </Step>
</Steps>

## Core principles [#core-principles]

<Cards>
  <Card title="Offline-first">
    No telemetry, no crash reporting, no analytics. Nothing leaves the machine unless the
    user explicitly exports it. The optional HTTP API binds to `127.0.0.1` by default.
  </Card>

  <Card title="Two languages, no overlap">
    All **code and documentation are English**; all **end-user UI text is Mongolian**
    (Cyrillic). See the language conventions in the repo `AGENTS.md`.
  </Card>

  <Card title="Stable contracts">
    Four hard rules (IPC path-strings only, no UI freeze on batch work, a single-JSON
    template format, an axum router that never owns its own lifecycle) are enforced on every
    PR.
  </Card>
</Cards>

## Where to start [#where-to-start]

<Cards>
  <Card href="/dev/architecture" title="Architecture">
    New to the codebase? Start here, then read the workspace map.
  </Card>

  <Card href="/dev/workspace" title="Workspace map">
    What each crate and app owns.
  </Card>

  <Card href="/dev/build" title="Build & run">
    Trying to build it? System prerequisites and commands per OS.
  </Card>

  <Card href="/dev/contributing" title="Contributing">
    Branching, commit conventions, language rules, and the PR checklist.
  </Card>

  <Card href="/dev/testing" title="Testing">
    How the Rust and frontend test suites are organized and run.
  </Card>

  <Card href="/dev/api" title="HTTP API reference">
    Integrating over HTTP? The `/v1/` surface.
  </Card>
</Cards>

<Callout type="info" title="Authoritative sources">
  The authoritative sources remain the repository's `AGENTS.md` files (one per crate) and
  the master plan at `.claude/PRPs/plans/shalgalt-omr-master.plan.md`. This site condenses
  them; where they disagree, the code wins for behaviour and the per-package `AGENTS.md`
  wins for module-shape decisions.
</Callout>
