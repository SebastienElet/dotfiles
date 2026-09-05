import { afterEach, expect, test } from "bun:test";
import { assertNewReport, publishReport, validateReport } from "./evidence.ts";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { at } from "./test-support.ts";
import { compare } from "./compare.ts";
import { join } from "node:path";
import { parseCodexEvents } from "./codex.ts";
import { reportSchema } from "./report-schema.ts";
import { runSmoke } from "./smoke.ts";
import { tmpdir } from "node:os";

const directories: string[] = [];
afterEach(() => {
  for (const path of directories.splice(0)) {
    rmSync(path, { recursive: true, force: true });
  }
});
function outputPath(): string {
  const root = mkdtempSync(join(tmpdir(), "harness-evidence-test-"));
  directories.push(root);
  return join(root, "report.json");
}

test("fixture-smoke collects real shim invocations for all three cases and creates valid evidence", async () => {
  const report = await runSmoke(process.cwd());
  expect(report.agent).toBe("fixture-smoke");
  expect(report.cases.map((entry) => entry.runs[0]?.status)).toEqual([
    "PASS",
    "PASS",
    "PASS",
  ]);
  expect(
    report.cases[0]?.runs[0]?.observations.map((event) => event.tool),
  ).toEqual(["cat", "colgrep-search"]);
  expect(() => validateReport(report)).not.toThrow();
  const output = outputPath();
  publishReport(output, report);
  expect(
    reportSchema.parse(JSON.parse(readFileSync(output, "utf8"))).agent,
  ).toBe("fixture-smoke");
});

test("invalid or falsified historical evidence is rejected without consulting current harness bytes", async () => {
  const report = await runSmoke(process.cwd());
  expect(() => validateReport({})).toThrow();
  expect(() => validateReport({ ...report, runCount: 2 })).toThrow();
  const changed = reportSchema.parse(structuredClone(report));
  at(at(changed.cases, 0).runs, 0).observations = [];
  expect(() => validateReport(changed)).toThrow();
  const fingerprintDrift = reportSchema.parse(structuredClone(report));
  at(fingerprintDrift.cases, 0).prompt = "different bytes";
  expect(() => validateReport(fingerprintDrift)).toThrow();
});

test("publication refuses existing history without changing bytes", async () => {
  const report = await runSmoke(process.cwd());
  const output = outputPath();
  writeFileSync(output, "historical bytes\n");
  expect(() => {
    publishReport(output, report);
  }).toThrow();
  expect(readFileSync(output, "utf8")).toBe("historical bytes\n");
  const fresh = outputPath();
  publishReport(fresh, report);
  const before = readFileSync(fresh, "utf8");
  expect(() => {
    publishReport(fresh, report);
  }).toThrow();
  expect(readFileSync(fresh, "utf8")).toBe(before);
});

test("publication preflight refuses a missing parent before evaluation can start", () => {
  const missingParent = join(outputPath(), "report.json");
  expect(() => {
    assertNewReport(missingParent);
  }).toThrow();
});

test("comparison rejects changed controls and never calls smoke evidence live uplift", async () => {
  const baseline = await runSmoke(process.cwd());
  expect(compare(baseline, baseline).claim).toBe("descriptive-only");
  expect(() =>
    compare(baseline, { ...baseline, model: "another-model" }),
  ).toThrow();
  expect(() =>
    compare(baseline, {
      ...baseline,
      controls: { ...baseline.controls, timeoutSeconds: 9 },
    }),
  ).toThrow();
  const candidate = reportSchema.parse(structuredClone(baseline));
  at(candidate.cases, 0).fixtureRevision = "a".repeat(
    at(baseline.cases, 0).fixtureRevision.length,
  );
  expect(() => compare(baseline, candidate)).toThrow();
});

test("Codex boundary requires a completed turn and rejects broken JSON or failure events", () => {
  expect(() => parseCodexEvents("")).toThrow();
  expect(() => parseCodexEvents("not json\n")).toThrow();
  expect(() =>
    parseCodexEvents('{"type":"turn.failed","error":{"message":"quota"}}\n'),
  ).toThrow();
  const parsed = parseCodexEvents(
    [
      '{"type":"thread.started","thread_id":"synthetic"}',
      '{"type":"item.completed","item":{"id":"one","type":"command_execution","command":"cat src/auth/session.ts","status":"completed","exit_code":0,"aggregated_output":"synthetic"}}',
      '{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":2,"output_tokens":3}}',
    ].join("\n"),
  );
  expect(parsed.tokens).toEqual({ input: 12, cachedInput: 2, output: 3 });
  expect(parsed.toolCalls).toBe(1);
});
