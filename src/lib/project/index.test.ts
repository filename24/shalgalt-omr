import { describe, expect, test, vi } from "vitest";

// assembleProject reaches into the DB + Tauri fs layers at import time; stub them
// so the test exercises only the pure manifest-assembly logic.
vi.mock("@tauri-apps/plugin-fs", () => ({ readTextFile: vi.fn() }));
vi.mock("$lib/db/jobs", () => ({
  getJobById: vi.fn(),
  parseGradedSheets: vi.fn(() => []),
}));
vi.mock("$lib/db/templates", () => ({
  getTemplate: vi.fn(async () => ({ schema: { questions: [] } })),
  createTemplate: vi.fn(),
}));

import { assembleProject } from "./index";

/**
 * The Rust writer rejects any archive where `manifest.encrypted` disagrees with
 * whether a passphrase was supplied, so the manifest flag must be derived purely
 * from the passphrase. These tests lock that contract.
 */
describe("assembleProject — manifest.encrypted", () => {
  test("is false when no passphrase is given", async () => {
    const { manifestJson } = await assembleProject({ templateId: 1, title: "T" });
    expect(JSON.parse(manifestJson).encrypted).toBe(false);
  });

  test("is true when a passphrase is given", async () => {
    const { manifestJson } = await assembleProject({
      templateId: 1,
      title: "T",
      passphrase: "hunter2",
    });
    expect(JSON.parse(manifestJson).encrypted).toBe(true);
  });

  test("treats an empty passphrase as no encryption", async () => {
    const { manifestJson } = await assembleProject({
      templateId: 1,
      title: "T",
      passphrase: "",
    });
    expect(JSON.parse(manifestJson).encrypted).toBe(false);
  });
});
