import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";

const project = join(import.meta.dirname, "..");

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
        path === "docs/codegraph.md" ||
        path === "docs/codegraph-validation.md",
    ),
  ).toEqual([]);

  const makefile = readFileSync(join(project, "Makefile"), "utf8");
  expect(makefile.toLowerCase()).not.toContain("codegraph");
  expect(
    readFileSync(join(project, "home/.config/git/ignore"), "utf8"),
  ).not.toContain(".codegraph/");
});
