#!/usr/bin/env node
//
// fetch-binaries.mjs — cross-platform launcher for the native-binary fetch scripts.
//
// Dispatches to scripts/fetch-binaries.ps1 on Windows and scripts/fetch-binaries.sh
// everywhere else, so a single command (and the package.json `postinstall` /
// `setup:binaries` hooks plus the Tauri `beforeDevCommand` / `beforeBuildCommand`) works
// on every developer OS.
//
// Behaviour:
//   - Skips automatically in CI (process.env.CI) — our GitHub workflows populate
//     apps/desktop/resources/ explicitly before building, so a postinstall download there
//     would be redundant and would add a network dependency to jobs that don't need it.
//   - Skips when SHALGALT_SKIP_FETCH_BINARIES is set, for offline-first setups.
//   - Non-fatal: a download failure (e.g. offline) warns but never breaks `pnpm install`.
//     The underlying scripts are idempotent and skip files that already exist.
//
// Extra args are forwarded to the platform script (e.g. `-Force` / `--force`).
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);

if (process.env.CI || process.env.SHALGALT_SKIP_FETCH_BINARIES) {
  console.log(
    "[fetch-binaries] skipped (CI or SHALGALT_SKIP_FETCH_BINARIES set). " +
      "Run scripts/fetch-binaries.{ps1,sh} manually if resources/ needs populating.",
  );
  process.exit(0);
}

const isWindows = process.platform === "win32";

function runWindows() {
  const ps1 = join(here, "fetch-binaries.ps1");
  const psArgs = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", ps1, ...args];
  let res = spawnSync("pwsh", psArgs, { stdio: "inherit" });
  // Older Windows boxes ship only Windows PowerShell (`powershell`), not `pwsh`.
  if (res.error && res.error.code === "ENOENT") {
    res = spawnSync("powershell", psArgs, { stdio: "inherit" });
  }
  return res;
}

function runUnix() {
  const sh = join(here, "fetch-binaries.sh");
  return spawnSync("bash", [sh, ...args], { stdio: "inherit" });
}

const res = isWindows ? runWindows() : runUnix();

if (res.error) {
  console.warn(`[fetch-binaries] could not launch the fetch script: ${res.error.message}`);
  console.warn("[fetch-binaries] continuing — run scripts/fetch-binaries.{ps1,sh} manually later.");
  process.exit(0);
}

if (res.status && res.status !== 0) {
  console.warn(`[fetch-binaries] fetch script exited with code ${res.status} — continuing.`);
  console.warn("[fetch-binaries] the build will fail later if resources/ is missing the runtime libraries.");
  process.exit(0);
}

process.exit(0);
