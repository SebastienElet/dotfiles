type Agent = "codex" | "claude" | "cursor";
type AgentCondition =
  | "admission"
  | "contradiction"
  | "control"
  | "proposal"
  | "relevant"
  | "sensitive"
  | "unavailable"
  | "unrelated";
type ProcessOutput = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
  cacheAbsentBefore?: boolean;
  cacheCompletedBeforeModel?: boolean;
  cachePath?: string;
  runtimeTrace?: string;
  traceCompletedBeforeModel?: boolean;
  traceAbsent?: boolean;
}>;
type ProcessRequest = Readonly<{
  acceptedExitCodes?: readonly number[];
  command: readonly string[];
  cwd?: string;
  environment: Readonly<NodeJS.ProcessEnv>;
  stdin?: string;
  timeoutMilliseconds: number;
}>;

async function runEvaluationProcess(
  command: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv>,
  timeoutMilliseconds: number,
): Promise<ProcessOutput> {
  return runManagedProcess({ command, environment, timeoutMilliseconds });
}

async function runManagedProcess(request: ProcessRequest): Promise<ProcessOutput> {
  const child = Bun.spawn([...request.command], {
    ...(request.cwd === undefined ? {} : { cwd: request.cwd }),
    env: { ...process.env, ...request.environment },
    stdin: request.stdin === undefined ? "ignore" : "pipe",
    stderr: "pipe",
    stdout: "pipe",
  });
  const input = child.stdin;
  if (request.stdin !== undefined && input !== undefined) {
    input.write(request.stdin);
    input.end();
  }
  const guard = superviseTimeout(child, request.timeoutMilliseconds);
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]).finally(guard.clear);
  await guard.finish();
  if (guard.expired()) throw new Error(`process timed out after ${request.timeoutMilliseconds}ms`);
  if (!(request.acceptedExitCodes ?? [0]).includes(exitCode)) {
    throw new Error(`process exit ${exitCode} (${diagnosticClass(stderr)})`);
  }
  return { exitCode, stderr, stdout };
}

async function runManagedProcessToFile(
  command: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv>,
  destination: string,
  timeoutMilliseconds: number,
): Promise<void> {
  const child = Bun.spawn([...command], {
    env: { ...process.env, ...environment },
    stderr: "pipe",
    stdout: Bun.file(destination),
  });
  const guard = superviseTimeout(child, timeoutMilliseconds);
  const [exitCode, stderr] = await Promise.all([
    child.exited,
    new Response(child.stderr).text(),
  ]).finally(guard.clear);
  await guard.finish();
  if (guard.expired()) throw new Error(`process timed out after ${timeoutMilliseconds}ms`);
  if (exitCode !== 0) {
    throw new Error(`process exit ${exitCode} (${diagnosticClass(stderr)})`);
  }
}

async function runObservedProcess(
  command: readonly string[],
  environment: NodeJS.ProcessEnv,
  cachePath: string,
  cacheReady: () => Promise<boolean>,
  timeoutMilliseconds: number,
): Promise<ProcessOutput> {
  const cacheAbsentBefore = !(await cacheReady());
  const child = Bun.spawn([...command], { env: environment, stderr: "pipe", stdout: "pipe" });
  const reader = child.stdout.getReader();
  const decoder = new TextDecoder();
  let stdout = "";
  let cacheCompletedBeforeModel = false;
  const guard = superviseTimeout(child, timeoutMilliseconds);
  while (true) {
    const chunk = await reader.read();
    if (chunk.done) break;
    if (!cacheCompletedBeforeModel && (await cacheReady())) cacheCompletedBeforeModel = true;
    const text = decoder.decode(chunk.value, { stream: true });
    stdout += text;
  }
  stdout += decoder.decode();
  const [exitCode, stderr] = await Promise.all([child.exited, new Response(child.stderr).text()]);
  guard.clear();
  await guard.finish();
  if (guard.expired()) throw new Error(`process timed out after ${timeoutMilliseconds}ms`);
  if (exitCode !== 0) throw new Error(`process exit ${exitCode} (${diagnosticClass(stderr)})`);
  return { cacheAbsentBefore, cacheCompletedBeforeModel, cachePath, exitCode, stderr, stdout };
}

async function runTraceObservedProcess(
  command: readonly string[],
  environment: NodeJS.ProcessEnv,
  tracePath: string,
  agent: Agent,
  condition: AgentCondition,
  modelNonce: string | undefined,
  timeoutMilliseconds: number,
  cache?: Readonly<{ path: string; ready: () => Promise<boolean> }>,
  cwd?: string,
): Promise<ProcessOutput> {
  const cacheAbsentBefore = cache === undefined ? undefined : !(await cache.ready());
  const child = Bun.spawn([...command], {
    ...(cwd === undefined ? {} : { cwd }),
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  const reader = child.stdout.getReader();
  const decoder = new TextDecoder();
  let stdout = "";
  let pending = "";
  let modelObserved = false;
  let traceCompletedBeforeModel = false;
  let cacheCompletedBeforeModel = false;
  const guard = superviseTimeout(child, timeoutMilliseconds);
  while (true) {
    const chunk = await reader.read();
    if (chunk.done) break;
    const traceCompleted = await traceHasCompletion(tracePath);
    const cacheCompleted = cache === undefined ? false : await cache.ready();
    const text = decoder.decode(chunk.value, { stream: true });
    stdout += text;
    pending += text;
    const lines = pending.split("\n");
    pending = lines.pop() ?? "";
    if (!modelObserved && modelNonce !== undefined) {
      modelObserved = lines.some((line) => nonceModelLine(environment, line, modelNonce));
      if (modelObserved) {
        traceCompletedBeforeModel = traceCompleted;
        cacheCompletedBeforeModel = cacheCompleted;
      }
    }
  }
  stdout += decoder.decode();
  const [exitCode, stderr] = await Promise.all([child.exited, new Response(child.stderr).text()]);
  guard.clear();
  await guard.finish();
  if (guard.expired()) {
    throw new Error(`${agent}:${condition}:process timed out after ${timeoutMilliseconds}ms`);
  }
  if (exitCode !== 0 || agentStreamFailed(stdout)) {
    throw conditionedAgentFailure(agent, condition, stdout, stderr);
  }
  const runtimeTrace = await readFile(tracePath, "utf8").catch(() => "");
  return {
    ...(cache === undefined || cacheAbsentBefore === undefined
      ? {}
      : { cacheAbsentBefore, cacheCompletedBeforeModel, cachePath: cache.path }),
    exitCode,
    runtimeTrace,
    stderr,
    stdout,
    traceAbsent: runtimeTrace === "",
    traceCompletedBeforeModel,
  };
}

async function traceHasCompletion(path: string): Promise<boolean> {
  const trace = await readFile(path, "utf8").catch(() => "");
  return trace.split("\n").some((line) => line.includes('"event":"completed"'));
}

function nonceModelLine(environment: NodeJS.ProcessEnv, line: string, nonce: string): boolean {
  const agent = environment.AGENT_MEMORY_EVAL_AGENT;
  if (agent !== "codex" && agent !== "claude" && agent !== "cursor") return false;
  try {
    const event: unknown = JSON.parse(line);
    if (!isRecord(event)) return false;
    return modelText(agent, event).includes(nonce);
  } catch {
    return false;
  }
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export {
  runEvaluationProcess,
  runManagedProcess,
  runManagedProcessToFile,
  runObservedProcess,
  runTraceObservedProcess,
};
export type { Agent, AgentCondition, ProcessOutput };
import { readFile } from "node:fs/promises";
import { modelText } from "./agent-memory-eval-contract.ts";
import {
  agentStreamFailed,
  conditionedAgentFailure,
  diagnosticClass,
} from "./agent-memory-eval-diagnostics.ts";
import { superviseTimeout } from "./agent-memory-eval-supervision.ts";
