import { mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { cacheContainsFixture } from "./agent-memory-eval-cache.ts";
import {
  runEvaluationProcess,
  runTraceObservedProcess,
} from "./agent-memory-eval-process.ts";
import type { Agent, AgentCondition, ProcessOutput } from "./agent-memory-eval-process.ts";
import { normalizeAgentVersion } from "./agent-memory-eval-claude.ts";
import {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  makeSourceUnavailable,
  runtimeSha,
  withEvaluationFixture,
} from "./agent-memory-eval-fixture.ts";
import {
  buildAgentCommand,
  claudeTemporaryDirectory,
  installCredential,
  withCursorAuthentication,
} from "./agent-memory-eval-auth.ts";
import {
  registerTemporaryPath,
  unregisterTemporaryPath,
} from "./agent-memory-eval-supervision.ts";
import { runReplicate } from "./agent-memory-eval-runner.ts";
import type { ReplicateResult } from "./agent-memory-eval-runner.ts";
import {
  assertCandidateIdentity,
  evaluatorCandidateSha,
  mergeReceipt,
  renderNormalizedReport,
} from "./agent-memory-eval-report.ts";
import type { AgentReceipt } from "./agent-memory-eval-report.ts";
type CliArguments = Readonly<{ agent: Agent; replicates: number }>;
const project = resolve(import.meta.dir, "..");
const processTimeoutMilliseconds = 120_000;
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
function parseArguments(arguments_: readonly string[]): CliArguments {
  const agentIndex = arguments_.indexOf("--agent");
  const replicateIndex = arguments_.indexOf("--replicates");
  const agent = arguments_[agentIndex + 1];
  const replicates = Number(arguments_[replicateIndex + 1]);
  if (!(["codex", "claude", "cursor"] as const).includes(agent as Agent)) {
    throw new Error("--agent must be codex, claude, or cursor");
  }
  if (!Number.isSafeInteger(replicates) || replicates < 1) {
    throw new Error("--replicates must be a positive integer");
  }
  return { agent: agent as Agent, replicates };
}
async function runAgent(
  agent: Agent,
  repository: string,
  environment: NodeJS.ProcessEnv,
  condition: AgentCondition,
  prompt: string,
  traceRoot: string,
  tracePath: string,
  nonce?: string,
  store?: string,
): Promise<ProcessOutput> {
  environment = {
    ...environment,
    AGENT_MEMORY_EVAL_AGENT: agent,
    AGENT_MEMORY_EVAL_ROOT: traceRoot,
    AGENT_MEMORY_EVAL_TRACE: tracePath,
  };
  if (condition === "control") {
    const isolated = join(environment.HOME ?? "", "control-home");
    await mkdir(isolated, { recursive: true, mode: 0o700 });
    environment = { ...environment, CODEX_HOME: join(isolated, ".codex"), HOME: isolated };
    await installCredential(agent, isolated, environment);
  }
  const command = buildAgentCommand(agent, traceRoot, repository, condition, prompt);
  const claudeTemporary = agent === "claude" ? claudeTemporaryDirectory(repository) : undefined;
  if (claudeTemporary !== undefined) registerTemporaryPath(claudeTemporary);
  try {
    if (agent === "cursor") {
      return await withCursorAuthentication(environment, (authenticated) =>
      runTraceObservedProcess(
        command,
        authenticated,
        tracePath,
        agent,
        condition,
        nonce,
        processTimeoutMilliseconds,
        condition === "relevant" && store !== undefined
          ? {
              path: join(store, "oracle-cache.json"),
              ready: () =>
                cacheContainsFixture(join(store, "oracle-cache.json"), "controller profile"),
            }
          : undefined,
        repository,
      ),
      );
    }
    return await runTraceObservedProcess(
    command,
    environment,
    tracePath,
    agent,
    condition,
    nonce,
    processTimeoutMilliseconds,
    condition === "relevant" && store !== undefined
      ? {
          path: join(store, "oracle-cache.json"),
          ready: () =>
            cacheContainsFixture(join(store, "oracle-cache.json"), "controller profile"),
        }
      : undefined,
    repository,
    );
  } finally {
    if (claudeTemporary !== undefined) {
      await rm(claudeTemporary, { force: true, recursive: true });
      unregisterTemporaryPath(claudeTemporary);
    }
  }
}

async function agentVersion(agent: Agent, environment: NodeJS.ProcessEnv): Promise<string> {
  const binary = agent === "cursor" ? "cursor-agent" : agent;
  const output =
    agent === "cursor"
      ? await withCursorAuthentication(environment, (authenticated) =>
          runEvaluationProcess([binary, "--version"], authenticated, 10_000),
        )
      : await runEvaluationProcess([binary, "--version"], environment, 10_000);
  return normalizeAgentVersion(agent, output.stdout);
}

async function main(): Promise<void> {
  const arguments_ = parseArguments(process.argv.slice(2));
  const initialCandidateSha = await evaluatorCandidateSha(project);
  const initialRuntimeSha = await runtimeSha();
  const results: ReplicateResult[] = [];
  let failure: unknown;
  try {
    for (let replicate = 1; replicate <= arguments_.replicates; replicate += 1) {
      results.push(await runReplicate(arguments_.agent, replicate, { agentVersion, runAgent }));
    }
  } catch (error) {
    failure = error;
  }
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
    results.length !== arguments_.replicates ||
    Object.values(counts).some((count) => count !== arguments_.replicates);
  const errorClass = failure === undefined && incomplete ? "capability_failure" : classifyFailure(failure);
  const receipt = await mergeReceipt(
    join(tmpdir(), "agent-memory-eval-receipts", `${initialCandidateSha}.json`),
    initialCandidateSha,
    initialRuntimeSha,
    agentReceipt(arguments_, results, errorClass),
  );
  await writeFile(
    join(project, "docs", "memory-governance-validation.md"),
    renderNormalizedReport(receipt),
  );
  if (failure !== undefined) throw failure;
  if (incomplete) throw new Error("capabilities below required replicate count");
}

function capabilityCounts(results: readonly ReplicateResult[]): Record<string, number> {
  return Object.fromEntries(
    expectedCapabilities.map((capability) => [
      capability,
      results.filter((result) =>
        result.capabilities.find((candidate) => candidate.capability === capability)?.passed,
      ).length,
    ]),
  );
}

function agentReceipt(
  arguments_: CliArguments,
  results: readonly ReplicateResult[],
  errorClass: string | undefined,
): AgentReceipt {
  const status =
    errorClass === undefined ? "complete" : errorClass === "usage_limit" ? "blocked" : "failed";
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

function classifyFailure(error: unknown): string | undefined {
  if (error === undefined) return undefined;
  const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
  if (message.includes("usage_limit") || message.includes("monthly usage limit")) return "usage_limit";
  if (message.includes("authentication")) return "authentication_unavailable";
  if (message.includes("timed out")) return "timeout";
  if (message.includes("cleanup failed")) return "cleanup_failure";
  return "evaluation_failure";
}

if (import.meta.main) {
  main().catch((error: unknown) => {
    process.stderr.write(`${error instanceof Error ? error.message : "evaluation failed"}\n`);
    process.exitCode = 1;
  });
}

export {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  buildAgentCommand,
  cacheContainsFixture,
  makeSourceUnavailable,
  runEvaluationProcess,
  withEvaluationFixture,
};
