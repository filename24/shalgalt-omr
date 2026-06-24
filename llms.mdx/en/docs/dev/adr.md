# Architecture decisions (https://filename24.github.io/shalgalt-omr/en/docs/dev/adr)



An &#x2A;*Architecture Decision Record (ADR)** is a short document that captures a significant
architectural or operational choice: the context, the alternatives considered, the decision,
and the consequences accepted. The value is in the *recorded reasoning* — so a future
contributor understands why the system looks the way it does without re-litigating settled
questions.

<Cards>
  <Card href="/adr" title="Browse all Architecture Decisions">
    The full set of accepted ADRs.
  </Card>
</Cards>

## When to write one [#when-to-write-one]

<Callout type="info" title="Write an ADR when…">
  * The decision is **hard to reverse** — e.g. choice of build toolchain, database engine, or
    IPC contract.
  * **Multiple credible options** exist and the trade-offs are non-obvious.
  * A future contributor would otherwise **re-litigate the same question**.

  If a change is small, local, and easily reversible, it does not need an ADR — just write
  the code.
</Callout>

## File naming [#file-naming]

ADRs live under `docs/adr/`, one file per decision:

```text title="ADR file naming"
docs/adr/NNNN-kebab-case-title.md
```

`NNNN` is a monotonically increasing four-digit number with **no gaps** — four digits keeps
the sort order stable past 100 entries.

## Template sections [#template-sections]

Use the structure of an existing ADR. Keep it short — the value is in the recorded
reasoning, not the prose volume.

<Steps>
  <Step>
    ### Status & metadata [#status--metadata]

    `Status` (Accepted / Superseded / …), `Date`, and `Deciders`.
  </Step>

  <Step>
    ### Context [#context]

    The forces and constraints that made this a real decision.
  </Step>

  <Step>
    ### Options [#options]

    The credible alternatives, with their trade-offs.
  </Step>

  <Step>
    ### Decision [#decision]

    What was chosen, and the rationale.
  </Step>

  <Step>
    ### Consequences [#consequences]

    What this commits the project to — including the costs accepted.
  </Step>

  <Step>
    ### Follow-up [#follow-up]

    Any deferred validation or related work the decision implies.
  </Step>
</Steps>

## Re-litigating a locked decision [#re-litigating-a-locked-decision]

<Callout type="warn" title="Locked decisions">
  Every accepted ADR is **locked**. Re-opening one requires a *new* ADR **plus** a
  master-plan update in the same PR. Don't quietly contradict an accepted decision in code —
  record the change. See the [contribution guide](/dev/contributing).
</Callout>
