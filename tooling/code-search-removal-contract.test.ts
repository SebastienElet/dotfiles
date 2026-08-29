import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";

const project = join(import.meta.dirname, "..");
const allowedHistoricalReferences = [
  "docs/adr/039-code-search.md",
  "docs/superpowers/plans/2026-08-29-code-search-replacement.md",
  "docs/superpowers/specs/2026-08-29-code-search-design.md",
  "tooling/code-search-ci-contract.test.ts",
  "tooling/code-search-removal-contract.test.ts",
];

test("keeps CodeGraph out of installation, runtime tooling, and agent skills", () => {
  const trackedFiles = Bun.spawnSync(["git", "ls-files"], {
    cwd: project,
    stderr: "pipe",
    stdout: "pipe",
  });
  expect(trackedFiles.exitCode).toBe(0);
  const paths = trackedFiles.stdout.toString().trimEnd().split("\n");
  expect(
    paths.filter(
      (path) =>
        path.startsWith("tooling/codegraph") ||
        path.startsWith("harness/skills/codegraph/") ||
        path === ".github/workflows/test-codegraph.yml" ||
        path === "docs/codegraph.md" ||
        path === "docs/codegraph-validation.md",
    ),
  ).toEqual([]);

  const makefile = readFileSync(join(project, "Makefile"), "utf8");
  expect(makefile.toLowerCase()).not.toContain("codegraph");
  expect(
    readFileSync(join(project, "home/.config/git/ignore"), "utf8"),
  ).not.toContain(".codegraph/");

  const references = Bun.spawnSync(
    ["git", "grep", "-Il", "-i", "codegraph", "--", "."],
    { cwd: project, stderr: "pipe", stdout: "pipe" },
  );
  expect(references.exitCode).toBe(0);
  expect(references.stdout.toString().trimEnd().split("\n")).toEqual(
    allowedHistoricalReferences,
  );
});
