import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";

const repositoryRoot = join(import.meta.dirname, "..");
const codeSearchWorkflow = readFileSync(
  join(repositoryRoot, ".github", "workflows", "test-code-search.yml"),
  "utf8",
);
const lintWorkflow = readFileSync(
  join(repositoryRoot, ".github", "workflows", "lint.yml"),
  "utf8",
);
const expectedPathFilters = 2;

test("code-search CI installs and exercises guarded ColGrep retrieval", () => {
  expect(
    codeSearchWorkflow.match(/^\s+- tooling\/colgrep-search\*$/gmu),
  ).toHaveLength(expectedPathFilters);
  expect(codeSearchWorkflow).toContain("colgrep --version");
  expect(codeSearchWorkflow).toContain(
    'make bundle-minimal "$HOME/.local/bin/colgrep-search"',
  );
  expect(codeSearchWorkflow).toContain(
    "bun test tooling/deployment-links.test.ts tooling/code-search-*.test.ts tooling/colgrep-search*.test.ts",
  );
  expect(codeSearchWorkflow).toContain(
    "COLGREP_INTEGRATION=1 bun test tooling/colgrep-search-integration.test.ts",
  );
  expect(codeSearchWorkflow.toLowerCase()).not.toContain("codegraph");
});

test("text CI formats every changed policy document", () => {
  for (const path of [
    "docs/adr/039-code-search.md",
    "docs/adr/README.md",
    "docs/code-search.md",
    "docs/superpowers/plans/2026-08-29-code-search-replacement.md",
    "docs/superpowers/specs/2026-08-29-code-search-design.md",
    "harness/skills/code-search/SKILL.md",
    "harness/skills/code-search/evals/trigger-queries.json",
  ]) {
    expect(countOccurrences(lintWorkflow, path)).toBe(1);
  }
});

function countOccurrences(content: string, value: string): number {
  return content.split(value).length - 1;
}
