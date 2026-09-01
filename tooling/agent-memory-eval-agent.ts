import type {
  Agent,
  AgentCondition,
  ProcessOutput,
} from "./agent-memory-eval-process.ts";
import {
  buildAgentCommand,
  claudeTemporaryDirectory,
  installCredential,
  withCursorAuthentication,
} from "./agent-memory-eval-auth.ts";
import { mkdir, rm } from "node:fs/promises";
import {
  registerTemporaryPath,
  unregisterTemporaryPath,
} from "./agent-memory-eval-supervision.ts";
import {
  runEvaluationProcess,
  runTraceObservedProcess,
} from "./agent-memory-eval-process.ts";
import { cacheContainsFixture } from "./agent-memory-eval-cache.ts";
import { join } from "node:path";
import { normalizeAgentVersion } from "./agent-memory-eval-claude.ts";

const agentVersionTimeoutMilliseconds = 10_000;
const processTimeoutMilliseconds = 120_000;
type RunAgentArguments = readonly [
  agent: Agent,
  repository: string,
  environment: Readonly<NodeJS.ProcessEnv>,
  condition: AgentCondition,
  prompt: string,
  traceRoot: string,
  tracePath: string,
  nonce?: string,
  store?: string,
];

async function runAgent(
  ...[
    agent,
    repository,
    environment,
    condition,
    prompt,
    traceRoot,
    tracePath,
    nonce,
    store,
  ]: RunAgentArguments
): Promise<ProcessOutput> {
  const agentEnvironment = await prepareEnvironment(
    agent,
    environment,
    condition,
    traceRoot,
    tracePath,
  );
  const command = buildAgentCommand(
    agent,
    traceRoot,
    repository,
    condition,
    prompt,
  );
  const claudeTemporary =
    agent === "claude" ? claudeTemporaryDirectory(repository) : undefined;
  if (claudeTemporary !== undefined) {
    registerTemporaryPath(claudeTemporary);
  }
  try {
    return await runAuthenticatedAgent({
      agent,
      command,
      condition,
      environment: agentEnvironment,
      nonce,
      repository,
      store,
      tracePath,
    });
  } finally {
    if (claudeTemporary !== undefined) {
      await rm(claudeTemporary, { force: true, recursive: true });
      unregisterTemporaryPath(claudeTemporary);
    }
  }
}

async function prepareEnvironment(
  ...[agent, environment, condition, traceRoot, tracePath]: readonly [
    Agent,
    Readonly<NodeJS.ProcessEnv>,
    AgentCondition,
    string,
    string,
  ]
): Promise<NodeJS.ProcessEnv> {
  const traced = {
    ...environment,
    AGENT_MEMORY_EVAL_AGENT: agent,
    AGENT_MEMORY_EVAL_ROOT: traceRoot,
    AGENT_MEMORY_EVAL_TRACE: tracePath,
  };
  if (condition !== "control") {
    return traced;
  }
  const isolated = join(environment.HOME ?? "", "control-home");
  await mkdir(isolated, { recursive: true, mode: 0o700 });
  const controlled = {
    ...traced,
    CODEX_HOME: join(isolated, ".codex"),
    HOME: isolated,
  };
  await installCredential(agent, isolated, controlled);
  return controlled;
}

type AgentRequest = Readonly<{
  agent: Agent;
  command: readonly string[];
  condition: AgentCondition;
  environment: Readonly<NodeJS.ProcessEnv>;
  nonce: string | undefined;
  repository: string;
  store: string | undefined;
  tracePath: string;
}>;

function runAuthenticatedAgent(request: AgentRequest): Promise<ProcessOutput> {
  if (request.agent === "cursor") {
    return withCursorAuthentication(request.environment, (authenticated) =>
      runObservedAgent(request, authenticated),
    );
  }
  return runObservedAgent(request, request.environment);
}

function runObservedAgent(
  request: AgentRequest,
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<ProcessOutput> {
  return runTraceObservedProcess(
    request.command,
    environment,
    request.tracePath,
    request.agent,
    request.condition,
    request.nonce,
    processTimeoutMilliseconds,
    request.condition === "relevant" && request.store !== undefined
      ? {
          path: join(request.store, "oracle-cache.json"),
          ready: () =>
            cacheContainsFixture(
              join(request.store ?? "", "oracle-cache.json"),
              "controller profile",
            ),
        }
      : undefined,
    request.repository,
  );
}

async function agentVersion(
  agent: Agent,
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<string> {
  const binary = agent === "cursor" ? "cursor-agent" : agent;
  const output =
    agent === "cursor"
      ? await withCursorAuthentication(environment, (authenticated) =>
          runEvaluationProcess(
            [binary, "--version"],
            authenticated,
            agentVersionTimeoutMilliseconds,
          ),
        )
      : await runEvaluationProcess(
          [binary, "--version"],
          environment,
          agentVersionTimeoutMilliseconds,
        );
  return normalizeAgentVersion(agent, output.stdout);
}

export { agentVersion, runAgent };
