---
title: ADR 0006 — Light theme & comfort typography
description: Make light the default theme via mode-watcher with dark opt-in, and add a one-toggle comfortable type scale driven by CSS variables.
---

<Callout type="success" title="Accepted · 2026-05-08">
  **Deciders:** filename24 · **Implements:** P2-09 / P2-10
</Callout>

## Context

The primary persona (master plan §1) is a Mongolian K–12 teacher who is **not** a
software professional. Two empirical constraints shaped the visual direction:

1. Classroom monitors and projectors are usually mid-grade IPS or VA panels with
   washed-out blacks, so a dark IDE chrome reads as low-contrast murk.
2. Older teachers — the median age of our beta cohort is ~52 — read 16 px UI text
   at a noticeably slower rate than 18 px text on the same hardware.

P0 shipped the app dark-only. We need light as the default, dark as opt-in, **no**
high-contrast mode (would force a third palette pair), and a one-toggle comfort
mode that bumps the entire scale a notch without per-component CSS.

## Options considered for theme

| Option | Implementation cost | Persistence | Per-screen drift risk |
| ------ | ------------------- | ----------- | --------------------- |
| Hand-rolled theme controller (`window.matchMedia` + manual class flip) | Medium | Need to hand-roll `localStorage` glue | High |
| **`mode-watcher` (already a dev dep)** | **Low** | **Built-in** | **Low** |
| CSS `@media (prefers-color-scheme)` only | Lowest | None — user can't override | High |

## Options considered for comfort sizing

| Option | Per-component cost | Toggle latency | Tailwind utility integrity |
| ------ | ------------------ | -------------- | -------------------------- |
| Add a `comfortable:` Tailwind variant; rewrite every component | High | Instant | Best |
| **`data-comfort` attribute on `<html>` + CSS variables on `:root[data-comfort=…]`** | **None — body font-size cascades** | **Instant** | **Acceptable** |
| Toggle a stylesheet at runtime | Medium | Flicker on swap | Acceptable |

## Decision

<Callout type="info" title="Decision">
  Adopt **`mode-watcher` with `defaultMode="light"`** for theming, and a
  **`data-comfort` attribute + CSS-variable type scale** for the comfortable-typography
  toggle.
</Callout>

**Theme (P2-09)**

- Mount `<ModeWatcher defaultMode="light" />` in `src/routes/+layout.svelte`.
- Move the dark palette under `.dark`; everything in `:root` now defines the light
  palette. Both legacy `--color-*` and the shadcn surface (`--background`, `--card`,
  `--primary`, plus new `--success` and `--warning`) gain light + dark values.
- `ThemeToggle.svelte` exposes light / dark / system via `setMode(...)` from
  `mode-watcher`. Mounted in the sidebar footer.
- Color tokens use OKLCH so contrast ratios stay predictable across hue rotations.
  Body text on background: AA (4.5:1+) in both modes; primary button on
  `--primary-foreground`: AAA (7:1+) in both modes.

**Comfort typography (P2-10)**

- Define `--text-{xs,sm,base,lg,xl,2xl}`, `--button-h`, `--hit-min` on `:root`
  (default scale: 12 / 14 / 16 / 18 / 22 / 28 px; 40 px button height).
- `:root[data-comfort="comfortable"]` overrides every token one notch up
  (14 / 16 / 18 / 20 / 24 / 32 px; 48 px button height).
- Body inherits `font-size: var(--text-base)`, so the toggle reflows the entire
  app instantly without per-component edits.
- `comfortMode` runes-store mirrors the choice to `<html data-comfort=…>` and
  persists to `localStorage` under `shalgalt-omr.comfort-mode`.
- `ComfortToggle.svelte` lives next to `ThemeToggle.svelte` in the sidebar footer.
- `Button` component default size is **not** updated to `min-h-[var(--button-h)]`
  in this iteration. Every existing button currently relies on shadcn's
  `h-7 / h-8 / h-9` ladder; a global flip would shift dialogs and toolbars across
  the app at once. The `--button-h` token is in place so we can do the swap as a
  one-line edit when we have layout coverage.

## Consequences

**Positive**

- Light is the default for fresh installs; existing dev installs persist their
  prior choice through `mode-watcher`'s storage.
- Comfort mode is two clicks of CSS away from being applied to any new component
  via `text-base` / `min-h-[var(--button-h)]` utilities.
- `--success` and `--warning` tokens make it easier to retire ad-hoc green/yellow
  Tailwind utilities in P3 (grading) and P4 (file format).

**Negative**

- Existing buttons stay on the shadcn `h-8` ladder until the follow-up.
- Comfort mode does not currently scale icons inside buttons — they use the
  shadcn `size-4` Tailwind utility. Pragmatic tradeoff; reads fine in user
  testing because the body line-height grows around them.

## Follow-up

- Promote the `Button` default size to `min-h-[var(--button-h)]` once the editor
  toolbar layout is QA'd at 48 px.
- If comfort mode ever needs an "extra-comfortable" notch (20 → 24 px body),
  add `:root[data-comfort="extra-comfortable"]` instead of overloading the
  current attribute value.
