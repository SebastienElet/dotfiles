import { type Report, reportSchema } from "./report-schema.ts";
import {
  accessSync,
  constants,
  linkSync,
  lstatSync,
  readFileSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { evaluate } from "./oracle.ts";
import { fingerprint } from "./sources.ts";
import { randomUUID } from "node:crypto";

const jsonIndent = 2;

function validateReport(raw: unknown): Report {
  const report: Report = reportSchema.parse(raw);
  const identifiers = report.cases.map((entry) => entry.definition.id);
  if (new Set(identifiers).size !== identifiers.length) {
    throw new Error("Duplicate report case ID");
  }
  for (const entry of report.cases) {
    if (entry.runs.length !== report.runCount) {
      throw new Error("Run count mismatch");
    }
    if (fingerprint(entry.prompt) !== entry.promptFingerprint) {
      throw new Error("Prompt fingerprint mismatch");
    }
    if (
      "text" in entry.definition.prompt &&
      entry.definition.prompt.text !== entry.prompt
    ) {
      throw new Error("Inline prompt mismatch");
    }
    if (
      JSON.stringify(entry.definition.sources) !==
      JSON.stringify(
        entry.sources.map(({ path, heading }) => ({ path, heading })),
      )
    ) {
      throw new Error("Source reference mismatch");
    }
    for (const run of entry.runs) {
      if ((run.status === "INVALID") !== (run.error !== null)) {
        throw new Error("Invalid status/error combination");
      }
      if (
        run.status !== "INVALID" &&
        evaluate(entry.definition.oracle, run.observations) !== run.status
      ) {
        throw new Error("Oracle verdict mismatch");
      }
    }
  }
  return report;
}

function readReport(path: string): Report {
  return validateReport(JSON.parse(readFileSync(path, "utf8")));
}

function assertNewReport(path: string): void {
  try {
    lstatSync(path);
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      const parent = dirname(path);
      if (!statSync(parent).isDirectory()) {
        throw new Error(`Report parent is not a directory: ${parent}`, {
          cause: error,
        });
      }
      accessSync(parent, constants.W_OK);
      return;
    }
    throw error;
  }
  throw new Error(`Report already exists: ${path}`);
}

function publishReport(path: string, raw: unknown): void {
  const report = validateReport(raw);
  assertNewReport(path);
  const temporary = join(dirname(path), `.harness-report-${randomUUID()}`);
  writeFileSync(temporary, `${JSON.stringify(report, null, jsonIndent)}\n`, {
    flag: "wx",
    mode: 0o600,
  });
  try {
    linkSync(temporary, path);
  } finally {
    unlinkSync(temporary);
  }
}

export { validateReport, readReport, assertNewReport, publishReport };
