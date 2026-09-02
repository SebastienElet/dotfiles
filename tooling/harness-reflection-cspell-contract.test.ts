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
  "325e779d8583df4408c4a618f25a7c2e42702fe4d1dfd6fcf7626191b848f13f";
const expectedSuccessfulCallLogSha256 =
  "008cae927f8148f190afab900a009527c31d5cdb15f1e848b0d3fb438ed04c38";
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

test("propagates a CSpell lint failure", async () => {
  const workflow = await readFile(workflowPath, "utf8");
  const result = await runCspellGate(
    extractCspellJob(workflow),
    failingLintStatus,
  );
  expect(result.status).toBe(failingLintStatus);
});
