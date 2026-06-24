# ADR index (https://filename24.github.io/shalgalt-omr/mn/docs/dev/adr)



# Architecture Decision Records [#architecture-decision-records]

ADRs record significant architectural or operational choices, the alternatives considered,
and the consequences accepted. Write one when a decision is hard to reverse, when multiple
credible options exist with non-obvious trade-offs, or when a future contributor would
otherwise re-litigate the same question. The canonical files live in `docs/adr/`.

| ID   | Title                                                                       | Status   |
| ---- | --------------------------------------------------------------------------- | -------- |
| 0001 | Windows OpenCV install strategy                                             | Accepted |
| 0002 | PDF generator: `printpdf`                                                   | Accepted |
| 0003 | Cargo workspace + crate boundaries                                          | Accepted |
| 0004 | Generate TypeScript bindings from Rust via `ts-rs`                          | Accepted |
| 0005 | PDF generation IPC contract                                                 | Accepted |
| 0006 | Light-default theme + comfortable typography mode                           | Accepted |
| 0007 | Canvas wrapper + per-element layout modules in `shalgalt-pdf`               | Accepted |
| 0008 | Bubble label position: inside the circle                                    | Accepted |
| 0009 | Corner markers: ArUco `DICT_6X6_50`, IDs 0..3 (TL/TR/BR/BL)                 | Accepted |
| 0010 | Per-bubble fill measurement, decision bands, confidence formula             | Accepted |
| 0011 | `.shalgalt` project file: zip container with a plaintext manifest           | Accepted |
| 0012 | Optional `.shalgalt` encryption: `age` passphrase recipients, ASCII-armored | Accepted |
| 0013 | HTTP API: `/v1` versioning, `DataStore` seam, OpenAPI via `utoipa`          | Accepted |
| 0014 | Standalone server + shared `rusqlite` `DataStore`                           | Accepted |
| 0015 | API port: default 22345, desktop auto-fallback, env override                | Accepted |
| 0016 | Partial-marker homography recovery                                          | Accepted |
| 0017 | Distribution via GitHub Releases (`tauri-action`)                           | Accepted |
| 0018 | Opt-in `tauri-plugin-updater` with static `gh-pages` manifest               | Accepted |
| 0019 | Documentation site: Fumadocs, deployed to GitHub Pages                      | Accepted |

> The full text of each ADR (Status / Date / Deciders / Context / Options / Decision /
> Consequences / Follow-up) lives in `docs/adr/NNNN-*.md` in the repository. Re-litigating a
> locked decision requires a new ADR plus a master-plan update in the same PR.
