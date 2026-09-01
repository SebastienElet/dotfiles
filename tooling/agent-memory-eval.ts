import { agentVersion, runAgent } from "./agent-memory-eval-agent.ts";
import {
  assertCandidateIdentity,
  evaluatorCandidateSha,
  mergeReceipt,
  renderNormalizedReport,
} from "./agent-memory-eval-report.ts";
import { join, resolve } from "node:path";
import type { Agent } from "./agent-memory-eval-process.ts";
import type { AgentReceipt } from "./agent-memory-eval-report.ts";
import type { ReplicateResult } from "./agent-memory-eval-runner.ts";
import { runReplicate } from "./agent-memory-eval-runner.ts";
import { runtimeSha } from "./agent-memory-eval-fixture.ts";
import { tmpdir } from "node:os";
import { writeFile } from "node:fs/promises";

type CliArguments = Readonly<{ agent: Agent; replicates: number }>;
type EvaluationResults = Readonly<{
  failure: Error | undefined;
  results: readonly ReplicateResult[];
}>;
const argumentOffset = 2;
const project = resolve(import.meta.dir, "..");
const expectedCapabilities = [
  "durable_detection",
  "complete_proposal",
  "no_implicit_write",
  "authorized_admission",
  "stored",
  "fresh_retrieval",
  "proof_before_influence",
  "freshness_before_influence",
  "unrelated_not_injected",
  "sensitive_rejected",
  "rejection_redacted",
  "store_unchanged",
  "unavailable_omitted",
  "unavailable_no_mutation",
  "contradiction_invalidated",
] as const;
function parseArguments(values: readonly string[]): CliArguments {
  const agentIndex = values.indexOf("--agent");
  const replicateIndex = values.indexOf("--replicates");
  const agent = parseAgent(values[agentIndex + 1]);
  const replicates = Number(values[replicateIndex + 1]);
  if (!Number.isSafeInteger(replicates) || replicates < 1) {
    throw new Error("--replicates must be a positive integer");
  }
  return { agent, replicates };
}

function parseAgent(value: string | undefined): Agent {
  if (value === "codex" || value === "claude" || value === "cursor") {
    return value;
  }
  throw new Error("--agent must be codex, claude, or cursor");
}

async function main(): Promise<void> {
  const cliArguments = parseArguments(process.argv.slice(argumentOffset));
  const initialCandidateSha = await evaluatorCandidateSha(project);
  const initialRuntimeSha = await runtimeSha();
  const { failure, results } = await collectReplicates(cliArguments);
  const finalCandidateSha = await evaluatorCandidateSha(project);
  const finalRuntimeSha = await runtimeSha();
  assertCandidateIdentity(
    initialCandidateSha,
    initialRuntimeSha,
    finalCandidateSha,
    finalRuntimeSha,
  );
  const counts = capabilityCounts(results);
  const incomplete =
    results.length !== cliArguments.replicates ||
    Object.values(counts).some((count) => count !== cliArguments.replicates);
  const errorClass =
    failure === undefined && incomplete
      ? "capability_failure"
      : classifyFailure(failure);
  const receipt = await mergeReceipt(
    join(tmpdir(), "agent-memory-eval-receipts", `${initialCandidateSha}.json`),
    initialCandidateSha,
    initialRuntimeSha,
    agentReceipt(cliArguments, results, errorClass),
  );
  await writeFile(
    join(project, "docs", "memory-governance-validation.md"),
    renderNormalizedReport(receipt),
  );
  if (failure !== undefined) {
    throw failure;
  }
  if (incomplete) {
    throw new Error("capabilities below required replicate count");
  }
}

async function collectReplicates(
  cliArguments: CliArguments,
): Promise<EvaluationResults> {
  const results: ReplicateResult[] = [];
  try {
    for (
      let replicate = 1;
      replicate <= cliArguments.replicates;
      replicate += 1
    ) {
      results.push(
        await runReplicate(cliArguments.agent, replicate, {
          agentVersion,
          runAgent,
        }),
      );
    }
    return { failure: undefined, results };
  } catch (error) {
    return { failure: asError(error), results };
  }
}

function capabilityCounts(
  results: readonly ReplicateResult[],
): Record<string, number> {
  return Object.fromEntries(
    expectedCapabilities.map((capability) => [
      capability,
      results.filter(
        (result) =>
          result.capabilities.find(
            (candidate) => candidate.capability === capability,
          )?.passed === true,
      ).length,
    ]),
  );
}

function agentReceipt(
  arguments_: CliArguments,
  results: readonly ReplicateResult[],
  errorClass: string | undefined,
): AgentReceipt {
  const status = receiptStatus(errorClass);
  return {
    agent: arguments_.agent,
    capabilities: capabilityCounts(results),
    cleanup: errorClass === "cleanup_failure" ? "failed" : "complete",
    completedReplicates: results.length,
    ...(errorClass === undefined ? {} : { errorClass }),
    requestedReplicates: arguments_.replicates,
    status,
    version: results[0]?.version ?? "unavailable",
  };
}

function receiptStatus(errorClass: string | undefined): AgentReceipt["status"] {
  if (errorClass === undefined) {
    return "complete";
  }
  return errorClass === "usage_limit" ? "blocked" : "failed";
}

function classifyFailure(
  error: Readonly<Error> | undefined,
): string | undefined {
  if (error === undefined) {
    return undefined;
  }
  const message = error.message.toLowerCase();
  if (
    message.includes("usage_limit") ||
    message.includes("monthly usage limit")
  ) {
    return "usage_limit";
  }
  if (message.includes("authentication")) {
    return "authentication_unavailable";
  }
  if (message.includes("timed out")) {
    return "timeout";
  }
  if (message.includes("cleanup failed")) {
    return "cleanup_failure";
  }
  return "evaluation_failure";
}

function asError(error: unknown): Error {
  return error instanceof Error ? error : new Error("evaluation failed");
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : "evaluation failed"}\n`,
    );
    process.exitCode = 1;
  }
}

export { buildAgentCommand } from "./agent-memory-eval-auth.ts";
export { cacheContainsFixture } from "./agent-memory-eval-cache.ts";
export {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  makeSourceUnavailable,
  withEvaluationFixture,
} from "./agent-memory-eval-fixture.ts";
export { runEvaluationProcess } from "./agent-memory-eval-process.ts";
