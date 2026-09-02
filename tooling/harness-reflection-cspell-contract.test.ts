import { expect, test } from "bun:test";
import {
  extractCspellJob,
  runCspellGate,
  sha256,
} from "./harness-reflection-cspell-test-support.ts";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const workflowPath = resolve(repositoryRoot, ".github/workflows/lint.yml");
const expectedCspellJobSha256 =
  "589fb007030cd172c8fe33084a28f6e630d455d193054c39bfde76fe9f9e25b7";
const expectedSuccessfulCallLogSha256 =
  "2bcdbb09425a62d876cd324a72039cbd22dcfcd349208b5bd37753ca5b0ed567";
const failingLintStatus = 17;

test("pins the complete CSpell job", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  expect(sha256(extractCspellJob(workflow))).toBe(expectedCspellJobSha256);
  expect(sha256(extractCspellJob(workflow.replaceAll("\n", "\r\n")))).toBe(
    expectedCspellJobSha256,
  );
  expect(() => extractCspellJob(workflow.replace("\n", "\r"))).toThrow(
    "line endings",
  );
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

test("changes the fingerprint for any job mutation", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const job = extractCspellJob(workflow);
  expect(
    sha256(
      job.replace("    name: Text configuration\n", "    name: Mutated\n"),
    ),
  ).not.toBe(expectedCspellJobSha256);
});

test("executes the nominal CSpell block with exact argv", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const result = await runCspellGate(extractCspellJob(workflow));
  expect(result.status).toBe(0);
  expect(sha256(result.normalizedCallLog)).toBe(
    expectedSuccessfulCallLogSha256,
  );
});

test.each([
  "harness/skills/harness-reflection/evals/promotion-workflow-results.json",
  "docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md",
  "docs/superpowers/plans/2026-09-02-registre-invariants-harnais.md",
  "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
  "tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json",
] as const)("executes CSpell on promised text %s", async (path) => {
  const workflow = await readFile(workflowPath, "utf8");
  const result = await runCspellGate(extractCspellJob(workflow));

  expect(result.normalizedCallLog).toContain(`\t${path}`);
});

test("propagates a CSpell lint failure", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const result = await runCspellGate(
    extractCspellJob(workflow),
    failingLintStatus,
  );
  expect(result.status).toBe(failingLintStatus);
});
