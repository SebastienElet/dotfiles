import {
  cspellGateIsFailClosed,
  extractCspellInstallArgv,
  extractCspellJob,
  extractCspellLintArgv,
} from "./harness-reflection-cspell-test-support.ts";
import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const workflowPath = resolve(repositoryRoot, ".github/workflows/lint.yml");
const cspellConfigPrefixLength = 4;
const promisedTexts = [
  "harness/skills/harness-reflection/SKILL.md",
  "harness/skills/harness-reflection/references/invariant-registry.md",
  "harness/skills/harness-reflection/evals/trigger-queries.json",
  "harness/invariants/registry.json",
  "harness/skills/harness-reflection/evals/promotion-workflow-results.json",
  "docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md",
  "docs/superpowers/plans/2026-09-02-registre-invariants-harnais.md",
  "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
  "tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json",
] as const;

test("pins only the owned CSpell installation argv", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  expect(extractCspellInstallArgv(extractCspellJob(workflow))).toEqual([
    "npm",
    "install",
    "--global",
    "cspell@10.2.0",
    "@cspell/dict-fr-fr@2.3.2",
  ]);
});

test("refuses a missing or duplicate CSpell job", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  expect(() =>
    extractCspellJob(workflow.replace("  cspell:\n", "  spelling:\n")),
  ).toThrow("exactly one CSpell job");
  expect(() => extractCspellJob(`${workflow}\n  "cspell" :\n`)).toThrow(
    "exactly one CSpell job",
  );
});

test("keeps the CSpell lint command fail-closed", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const job = extractCspellJob(workflow);
  expect(cspellGateIsFailClosed(job)).toBe(true);
  expect(
    cspellGateIsFailClosed(job.replace("set -euo pipefail", "set +e")),
  ).toBe(false);
});

test("owns the CSpell config argv before every promised text", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const argv = extractCspellLintArgv(extractCspellJob(workflow));
  expect(argv.slice(0, cspellConfigPrefixLength)).toEqual([
    "cspell",
    "lint",
    "--config",
    '"$test_home/cspell.json"',
  ]);
  for (const path of promisedTexts) {
    expect(argv.filter((value) => value === path)).toHaveLength(1);
  }
});

test.each([...promisedTexts])(
  "exposes omission of promised text %s",
  async (path) => {
    const workflow = await readFile(workflowPath, "utf8");
    const mutant = extractCspellJob(workflow).replace(path, "omitted-text");
    expect(extractCspellLintArgv(mutant)).not.toContain(path);
  },
);
