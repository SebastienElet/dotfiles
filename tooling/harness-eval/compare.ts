import type { Report } from "./report-schema.ts";
import { validateReport } from "./evidence.ts";

type ControlledIdentity = Pick<
  Report,
  | "agent"
  | "agentVersion"
  | "model"
  | "runnerRevision"
  | "environment"
  | "controls"
  | "runCount"
> &
  Readonly<{
    cases: readonly Readonly<{
      definition: Report["cases"][number]["definition"];
      prompt: string;
      fixture: string;
    }>[];
  }>;
type Metrics = Readonly<{
  passRate: number;
  failures: number;
  invalid: number;
  meanTokens: number | null;
  meanToolCalls: number | null;
  meanDurationMs: number | null;
}>;
type Comparison = Readonly<{
  comparable: true;
  claim: "descriptive-only";
  baseline: Metrics;
  candidate: Metrics;
  passRateDelta: number;
  regressions: readonly Readonly<{ caseId: string; run: number }>[];
  harness: Readonly<{
    baseline: Report["harness"];
    candidate: Report["harness"];
  }>;
  limitations: readonly string[];
}>;

function controlledIdentity(report: Report): ControlledIdentity {
  return {
    agent: report.agent,
    agentVersion: report.agentVersion,
    model: report.model,
    runnerRevision: report.runnerRevision,
    environment: report.environment,
    controls: report.controls,
    runCount: report.runCount,
    cases: report.cases.map((entry) => ({
      definition: entry.definition,
      prompt: entry.promptFingerprint,
      fixture: entry.fixtureRevision,
    })),
  };
}

function metrics(report: Report): Metrics {
  const runs = report.cases.flatMap((entry) => entry.runs);
  const mean = (values: readonly (number | null)[]): number | null =>
    values.some((value) => value === null)
      ? null
      : values.reduce<number>((sum, value) => sum + (value ?? 0), 0) /
        values.length;
  return {
    passRate: runs.filter((run) => run.status === "PASS").length / runs.length,
    failures: runs.filter((run) => run.status === "FAIL").length,
    invalid: runs.filter((run) => run.status === "INVALID").length,
    meanTokens: mean(
      runs.map((run) =>
        run.tokens === null ? null : run.tokens.input + run.tokens.output,
      ),
    ),
    meanToolCalls: mean(runs.map((run) => run.toolCalls)),
    meanDurationMs: mean(runs.map((run) => run.durationMs)),
  };
}

function compare(baselineRaw: unknown, candidateRaw: unknown): Comparison {
  const baseline = validateReport(baselineRaw);
  const candidate = validateReport(candidateRaw);
  if (
    JSON.stringify(controlledIdentity(baseline)) !==
    JSON.stringify(controlledIdentity(candidate))
  ) {
    throw new Error(
      "Reports are not comparable: cases, prompts, fixtures, runner, model, environment, permissions or budgets differ",
    );
  }
  const before = metrics(baseline);
  const after = metrics(candidate);
  return {
    comparable: true,
    claim: "descriptive-only",
    baseline: before,
    candidate: after,
    passRateDelta: after.passRate - before.passRate,
    regressions: baseline.cases.flatMap((entry, index) =>
      entry.runs.flatMap((run, replicate) =>
        run.status === "PASS" &&
        candidate.cases[index]?.runs[replicate]?.status !== "PASS"
          ? [{ caseId: entry.definition.id, run: replicate + 1 }]
          : [],
      ),
    ),
    harness: { baseline: baseline.harness, candidate: candidate.harness },
    limitations: [
      "Matching recorded controls does not establish causality or statistical significance.",
      "Replicate positions are paired descriptively; Codex does not expose paired random seeds.",
      "INVALID runs remain in the denominator; fixture-smoke is not live behavioral evidence.",
    ],
  };
}

export { type Comparison, type Metrics, compare };
