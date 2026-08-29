import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";

const repositoryRoot = join(import.meta.dirname, "..");
const codegraphWorkflow = readFileSync(
  join(repositoryRoot, ".github", "workflows", "test-codegraph.yml"),
  "utf8",
);
const lintWorkflow = readFileSync(
  join(repositoryRoot, ".github", "workflows", "lint.yml"),
  "utf8",
);
const expectedPathFilters = 2;

test("CodeGraph CI installs and exercises guarded ColGrep retrieval", () => {
  expect(
    codegraphWorkflow.match(/^\s+- tooling\/colgrep-worktree\*$/gmu),
  ).toHaveLength(expectedPathFilters);
  expect(codegraphWorkflow).toContain("colgrep --version");
  expect(codegraphWorkflow).toContain(
    'make bundle-minimal codegraph-cli "$HOME/.local/bin/colgrep-worktree"',
  );
  expect(codegraphWorkflow).toContain(
    "bun test tooling/deployment-links.test.ts tooling/colgrep-worktree*.test.ts",
  );
  expect(codegraphWorkflow).toContain(
    "COLGREP_INTEGRATION=1 bun test tooling/colgrep-worktree-integration.test.ts",
  );
  expect(codegraphWorkflow).not.toContain("skill_contract_test.sh");
});

test("text CI formats every changed policy document", () => {
  for (const path of [
    "docs/adr/039-codegraph-recuperation-structurelle.md",
    "docs/adr/README.md",
    "docs/codegraph.md",
    "docs/superpowers/plans/2026-08-29-colgrep-worktree-routing.md",
    "docs/superpowers/specs/2026-08-29-recherche-worktrees-design.md",
    "harness/skills/codegraph/SKILL.md",
    "harness/skills/codegraph/evals/trigger-queries.json",
  ]) {
    expect(countOccurrences(lintWorkflow, path)).toBe(1);
  }
});

function countOccurrences(content: string, value: string): number {
  return content.split(value).length - 1;
}
