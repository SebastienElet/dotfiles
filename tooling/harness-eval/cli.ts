import { assertNewReport, publishReport, readReport } from "./evidence.ts";
import { validateEvaluations, validateEvidence } from "./validate.ts";
import type { SeriesOptions } from "./runner.ts";
import { compare } from "./compare.ts";
import { controlsSchema } from "./report-schema.ts";
import { parseArgs } from "node:util";
import { resolve } from "node:path";
import { z } from "zod";

const ARGUMENT_OFFSET = 2;
const JSON_INDENT = 2;
const MAX_RUNS = 10;
const runCountSchema = z.coerce.number().int().positive().max(MAX_RUNS);
type EvalOptions = Omit<SeriesOptions, "agent" | "agentVersion"> &
  Readonly<{ report: string }>;

function evalOptions(args: readonly string[]): EvalOptions {
  const { values } = parseArgs({
    args,
    options: {
      model: { type: "string" },
      only: { type: "string" },
      runs: { type: "string", default: "1" },
      report: { type: "string" },
      "timeout-seconds": { type: "string", default: "120" },
      "reasoning-effort": { type: "string", default: "low" },
      "variant-file": { type: "string" },
    },
  });
  const model = z
    .string()
    .regex(/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/u)
    .parse(values.model);
  const only = z.string().min(1).parse(values.only).split(",");
  const report = resolve(z.string().min(1).parse(values.report));
  const runs = runCountSchema.parse(values.runs);
  const controls = controlsSchema.parse({
    sandbox: "workspace-write",
    network: false,
    tools: "shell-with-synthetic-cat-rg-fd-colgrep-v1",
    timeoutSeconds: Number(values["timeout-seconds"]),
    reasoningEffort: values["reasoning-effort"],
    tokenBudget: null,
  });
  const variant = values["variant-file"];
  if (
    variant !== undefined &&
    !/^harness\/evals\/variants\/[a-z0-9-]+\.md$/u.test(variant)
  ) {
    throw new Error(
      "Variant must be a synthetic Markdown file under harness/evals/variants/",
    );
  }
  return {
    model,
    only,
    report,
    runs,
    controls,
    ...(variant === undefined ? {} : { variant }),
  };
}

function validateSelectedReports(
  repository: string,
  paths: readonly string[],
): string {
  if (paths.length === 0) {
    return validateEvidence(repository);
  }
  for (const path of paths) {
    readReport(path);
  }
  return `${paths.length} selected reports valid`;
}

async function evalAndPublish(
  repository: string,
  options: readonly string[],
): Promise<string> {
  const selected = evalOptions(options);
  assertNewReport(selected.report);
  const { runLive } = await import("./live.ts");
  const report = await runLive(repository, selected);
  publishReport(selected.report, report);
  if (
    report.cases.some((entry) =>
      entry.runs.some((run) => run.status !== "PASS"),
    )
  ) {
    process.exitCode = 1;
  }
  return `New report: ${selected.report}`;
}

async function dispatch(args: readonly string[]): Promise<string> {
  const [operation, ...options] = args;
  const repository = resolve(import.meta.dir, "../..");
  if (operation === "validate-evals" && options.length === 0) {
    return validateEvaluations(repository);
  }
  if (operation === "validate-evidence") {
    return validateSelectedReports(repository, options);
  }
  if (operation === "compare") {
    const [baseline, candidate] = z
      .tuple([z.string(), z.string()])
      .parse(options);
    return JSON.stringify(
      compare(readReport(baseline), readReport(candidate)),
      null,
      JSON_INDENT,
    );
  }
  if (operation === "fixture-smoke" && options.length === 0) {
    const { runSmoke } = await import("./smoke.ts");
    return JSON.stringify(await runSmoke(repository), null, JSON_INDENT);
  }
  if (operation === "eval") {
    return evalAndPublish(repository, options);
  }
  throw new Error(
    "Expected validate-evals, validate-evidence [reports], fixture-smoke, compare <baseline> <candidate>, or eval --model <id> --only <ids> --report <new-file>",
  );
}

async function main(args: readonly string[]): Promise<void> {
  process.stdout.write(`${await dispatch(args)}\n`);
}

if (import.meta.main) {
  try {
    await main(process.argv.slice(ARGUMENT_OFFSET));
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : "Harness command failed"}\n`,
    );
    process.exitCode = 1;
  }
}

export { main };
