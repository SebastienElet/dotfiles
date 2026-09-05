import type { Controls, Report, Run } from "./report-schema.ts";
import {
  type LoadedCase,
  fingerprint,
  loadCases,
  readSource,
} from "./sources.ts";
import {
  type PreparedFixture,
  collectObservations,
  prepareFixture,
} from "./fixture.ts";
import { readdirSync, rmSync } from "node:fs";
import { evaluate } from "./oracle.ts";
import { join } from "node:path";
import { validateReport } from "./evidence.ts";

const MAX_RUNS = 10;
type Execution = Omit<Run, "status" | "observations">;
type Executor = (
  fixture: PreparedFixture,
  prompt: string,
) => Promise<Execution> | Execution;
type SeriesOptions = Readonly<{
  agent: string;
  agentVersion: string;
  model: string;
  only: readonly string[];
  runs: number;
  controls: Controls;
  variant?: string;
}>;
type SeriesContext = Readonly<{
  repository: string;
  options: SeriesOptions;
  revision: string;
}>;
type EvaluatedCase = Readonly<{
  harness: Report["harness"];
  result: Report["cases"][number];
}>;

function runnerRevision(repository: string): string {
  const files = readdirSync(join(repository, "tooling/harness-eval"))
    .filter(
      (path) =>
        path.endsWith(".ts") &&
        !path.endsWith(".test.ts") &&
        path !== "test-support.ts",
    )
    .toSorted();
  return fingerprint(
    files
      .map(
        (path) =>
          `${path}\0${readSource(repository, `tooling/harness-eval/${path}`)}`,
      )
      .join("\0"),
  );
}

function gitRevision(repository: string): string {
  const result = Bun.spawnSync(["git", "-C", repository, "rev-parse", "HEAD"], {
    env: {
      PATH: "/usr/bin:/bin",
      GIT_CONFIG_NOSYSTEM: "1",
      GIT_CONFIG_GLOBAL: "/dev/null",
    },
  });
  if (result.exitCode !== 0) {
    throw new Error("Cannot identify harness Git revision");
  }
  return result.stdout.toString().trim();
}

async function observe(
  fixture: PreparedFixture,
  entry: LoadedCase,
  execute: Executor,
): Promise<Run> {
  const result = await execute(fixture, entry.prompt);
  try {
    const observations = collectObservations(fixture.observations);
    return {
      ...result,
      observations,
      status:
        result.error === null
          ? evaluate(entry.definition.oracle, observations)
          : "INVALID",
    };
  } catch {
    return {
      ...result,
      observations: [],
      status: "INVALID",
      error: "observation-invalid",
    };
  }
}

async function evaluateCase(
  context: SeriesContext,
  entry: LoadedCase,
  execute: Executor,
): Promise<EvaluatedCase> {
  const runs: Run[] = [];
  let identity: Report["harness"] | undefined = undefined;
  let fixtureRevision = "";
  for (let index = 0; index < context.options.runs; index += 1) {
    const fixture = prepareFixture(
      context.repository,
      entry,
      context.options.variant,
    );
    try {
      const harness = {
        gitRevision: context.revision,
        instructionFingerprint: fixture.instructionFingerprint,
        skillFingerprint: fixture.skillFingerprint,
        variant: context.options.variant ?? "Context Management",
      };
      if (
        identity !== undefined &&
        JSON.stringify(identity) !== JSON.stringify(harness)
      ) {
        throw new Error("Harness changed during evaluation");
      }
      identity = harness;
      ({ fixtureRevision } = fixture);
      runs.push(await observe(fixture, entry, execute));
    } finally {
      rmSync(fixture.root, { recursive: true, force: true });
    }
  }
  if (identity === undefined) {
    throw new Error("No evaluation runs");
  }
  return {
    harness: identity,
    result: {
      definition: entry.definition,
      sources: entry.sources,
      prompt: entry.prompt,
      promptFingerprint: entry.promptFingerprint,
      fixtureRevision,
      runs,
    },
  };
}

function selectCases(
  repository: string,
  options: SeriesOptions,
): readonly LoadedCase[] {
  const available = loadCases(repository);
  if (
    options.only.length === 0 ||
    new Set(options.only).size !== options.only.length ||
    options.only.some(
      (id) => !available.some((entry) => entry.definition.id === id),
    )
  ) {
    throw new Error("Explicit, unique known case selection required");
  }
  if (
    !Number.isInteger(options.runs) ||
    options.runs < 1 ||
    options.runs > MAX_RUNS
  ) {
    throw new Error("Runs must be between 1 and 10");
  }
  return available.filter((entry) =>
    options.only.includes(entry.definition.id),
  );
}

const limitations = [
  "Only Context Management and code-search are installed; this is not the full deployed harness.",
  "PATH shims observe supported commands, not internal skill loading or uninstrumented reads; bypasses can cause false negatives.",
  "Synthetic shims are not ColGrep quality or security tests; observations are not tamper-proof.",
  "Git revision plus content fingerprints identify the tested bytes, including uncommitted changes.",
  "Model is the requested ID; aliases may resolve differently. No statistical or causal uplift claim.",
  "No token ceiling is available; the wall-clock timeout bounds each run. Raw transcripts are discarded.",
] as const;

async function runSeries(
  repository: string,
  options: SeriesOptions,
  execute: Executor,
): Promise<Report> {
  const selected = selectCases(repository, options);
  const evaluated: EvaluatedCase[] = [];
  const revision = gitRevision(repository);
  const runner = runnerRevision(repository);
  for (const entry of selected) {
    evaluated.push(
      await evaluateCase({ repository, options, revision }, entry, execute),
    );
  }
  const [first] = evaluated;
  if (
    first === undefined ||
    evaluated.some(
      (entry) =>
        JSON.stringify(entry.harness) !== JSON.stringify(first.harness),
    ) ||
    runnerRevision(repository) !== runner
  ) {
    throw new Error("Evaluation sources changed during execution");
  }
  return validateReport({
    schemaVersion: 1,
    agent: options.agent,
    agentVersion: options.agentVersion,
    model: options.model,
    date: new Date().toISOString(),
    harness: first.harness,
    runnerRevision: runner,
    environment: {
      platform: process.platform,
      architecture: process.arch,
      bun: Bun.version,
    },
    controls: options.controls,
    runCount: options.runs,
    cases: evaluated.map((entry) => entry.result),
    limitations,
  });
}

export { runnerRevision, runSeries };
export type { Execution, Executor, SeriesOptions };
