import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";

const repositoryRoot = join(import.meta.dirname, "..");
const codeSearchWorkflow = readFileSync(
  join(repositoryRoot, ".github", "workflows", "test-code-search.yml"),
  "utf8",
);
const expectedPathFilters = 2;

test("code-search CI installs and exercises guarded ColGrep retrieval", () => {
  expect(
    codeSearchWorkflow.match(/^\s+- tooling\/colgrep-search\*$/gmu),
  ).toHaveLength(expectedPathFilters);
  expect(codeSearchWorkflow).toContain("colgrep --version");
  expect(codeSearchWorkflow).toContain("brew install lightonai/tap/colgrep");
  expect(codeSearchWorkflow).toContain(
    'make "$HOME/.local/bin/colgrep-search"',
  );
  expect(codeSearchWorkflow).toContain(
    "bun test tooling/deployment-links.test.ts tooling/code-search-*.test.ts tooling/colgrep-search*.test.ts",
  );
  expect(codeSearchWorkflow).toContain(
    "COLGREP_INTEGRATION=1 bun test tooling/colgrep-search-integration.test.ts",
  );
  expect(codeSearchWorkflow.toLowerCase()).not.toContain("codegraph");
});
