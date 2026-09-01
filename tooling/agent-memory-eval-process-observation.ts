import type {
  Agent,
  AgentCondition,
  ProcessOutput,
} from "./agent-memory-eval-process-types.ts";
import {
  agentStreamFailed,
  conditionedAgentFailure,
  diagnosticClass,
} from "./agent-memory-eval-diagnostics.ts";
import {
  nonceModelLine,
  readTrace,
  traceHasCompletion,
} from "./agent-memory-eval-process-trace.ts";
import { superviseTimeout } from "./agent-memory-eval-supervision.ts";

type Cache = Readonly<{ path: string; ready: () => Promise<boolean> }>;
type TraceRequest = Readonly<{
  agent: Agent;
  cache?: Cache | undefined;
  command: readonly string[];
  condition: AgentCondition;
  cwd?: string | undefined;
  environment: Readonly<NodeJS.ProcessEnv>;
  modelNonce: string | undefined;
  timeoutMilliseconds: number;
  tracePath: string;
}>;
type StreamObservation = Readonly<{
  cacheCompletedBeforeModel: boolean;
  stdout: string;
  traceCompletedBeforeModel: boolean;
}>;
type ObservedSuccessRequest = Readonly<{
  exitCode: number;
  guard: Readonly<{ expired: () => boolean }>;
  stderr: string;
  timeoutMilliseconds: number;
}>;
type TraceSuccessRequest = Readonly<{
  exitCode: number;
  guard: Readonly<{ expired: () => boolean }>;
  process: Readonly<TraceRequest>;
  stderr: string;
  stdout: string;
}>;
type TraceOutputRequest = Readonly<{
  cacheAbsentBefore: boolean | undefined;
  exitCode: number;
  observation: StreamObservation;
  process: Readonly<TraceRequest>;
  stderr: string;
}>;

async function runObservedProcess(
  ...[
    command,
    environment,
    cachePath,
    cacheReady,
    timeoutMilliseconds,
  ]: readonly [
    readonly string[],
    Readonly<NodeJS.ProcessEnv>,
    string,
    () => Promise<boolean>,
    number,
  ]
): Promise<ProcessOutput> {
  const cacheAbsentBefore = !(await cacheReady());
  const child = Bun.spawn([...command], {
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  const guard = superviseTimeout(child, timeoutMilliseconds);
  const stdout = await observeCacheOutput(child.stdout, cacheReady);
  const [exitCode, stderr] = await waitForObservedProcess(child, guard);
  assertObservedSuccess({ exitCode, guard, stderr, timeoutMilliseconds });
  return {
    cacheAbsentBefore,
    cacheCompletedBeforeModel: stdout.cacheCompleted,
    cachePath,
    exitCode,
    stderr,
    stdout: stdout.text,
  };
}

function runTraceObservedProcess(
  ...[
    command,
    environment,
    tracePath,
    agent,
    condition,
    modelNonce,
    timeoutMilliseconds,
    cache,
    cwd,
  ]: readonly [
    readonly string[],
    Readonly<NodeJS.ProcessEnv>,
    string,
    Agent,
    AgentCondition,
    string | undefined,
    number,
    (Cache | undefined)?,
    (string | undefined)?,
  ]
): Promise<ProcessOutput> {
  return runTraceProcess({
    agent,
    cache,
    command,
    condition,
    cwd,
    environment,
    modelNonce,
    timeoutMilliseconds,
    tracePath,
  });
}

async function runTraceProcess(
  request: Readonly<TraceRequest>,
): Promise<ProcessOutput> {
  const cacheAbsentBefore = await absentBefore(request.cache);
  const child = Bun.spawn([...request.command], {
    ...(request.cwd === undefined ? {} : { cwd: request.cwd }),
    env: request.environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  const guard = superviseTimeout(child, request.timeoutMilliseconds);
  const observation = await observeTraceOutput(child.stdout, request);
  const [exitCode, stderr] = await waitForObservedProcess(child, guard);
  assertTraceSuccess({
    exitCode,
    guard,
    process: request,
    stderr,
    stdout: observation.stdout,
  });
  return traceOutput({
    cacheAbsentBefore,
    exitCode,
    observation,
    process: request,
    stderr,
  });
}

async function observeCacheOutput(
  output: Readonly<ReadableStream<Uint8Array>>,
  cacheReady: () => Promise<boolean>,
): Promise<Readonly<{ cacheCompleted: boolean; text: string }>> {
  const reader = output.getReader();
  const decoder = new TextDecoder();
  let cacheCompleted = false;
  let text = "";
  for (
    let chunk = await reader.read();
    !chunk.done;
    chunk = await reader.read()
  ) {
    cacheCompleted ||= await cacheReady();
    text += decoder.decode(chunk.value, { stream: true });
  }
  return { cacheCompleted, text: text + decoder.decode() };
}

async function observeTraceOutput(
  output: Readonly<ReadableStream<Uint8Array>>,
  request: Readonly<TraceRequest>,
): Promise<StreamObservation> {
  const reader = output.getReader();
  const decoder = new TextDecoder();
  let pending = "";
  let stdout = "";
  let modelObserved = false;
  let cacheCompletedBeforeModel = false;
  let traceCompletedBeforeModel = false;
  for (
    let chunk = await reader.read();
    !chunk.done;
    chunk = await reader.read()
  ) {
    const text = decoder.decode(chunk.value, { stream: true });
    stdout += text;
    pending += text;
    const lines = pending.split("\n");
    pending = lines.pop() ?? "";
    if (!modelObserved && request.modelNonce !== undefined) {
      modelObserved = lines.some((line) =>
        nonceModelLine(request.environment, line, request.modelNonce ?? ""),
      );
      if (modelObserved) {
        cacheCompletedBeforeModel =
          request.cache === undefined ? false : await request.cache.ready();
        traceCompletedBeforeModel = await traceHasCompletion(request.tracePath);
      }
    }
  }
  return {
    cacheCompletedBeforeModel,
    stdout: stdout + decoder.decode(),
    traceCompletedBeforeModel,
  };
}

async function waitForObservedProcess(
  child: Readonly<{
    exited: Readonly<Promise<number>>;
    stderr: Readonly<ReadableStream<Uint8Array>>;
  }>,
  guard: Readonly<{ clear: () => void; finish: () => Promise<void> }>,
): Promise<readonly [number, string]> {
  try {
    return await Promise.all([child.exited, new Response(child.stderr).text()]);
  } finally {
    guard.clear();
    await guard.finish();
  }
}

function assertObservedSuccess(request: ObservedSuccessRequest): void {
  if (request.guard.expired()) {
    throw new Error(`process timed out after ${request.timeoutMilliseconds}ms`);
  }
  if (request.exitCode !== 0) {
    throw new Error(
      `process exit ${request.exitCode} (${diagnosticClass(request.stderr)})`,
    );
  }
}

function assertTraceSuccess(request: TraceSuccessRequest): void {
  if (request.guard.expired()) {
    throw new Error(
      `${request.process.agent}:${request.process.condition}:process timed out after ${request.process.timeoutMilliseconds}ms`,
    );
  }
  if (request.exitCode !== 0 || agentStreamFailed(request.stdout)) {
    throw conditionedAgentFailure(
      request.process.agent,
      request.process.condition,
      request.stdout,
      request.stderr,
    );
  }
}

async function absentBefore(
  cache: Cache | undefined,
): Promise<boolean | undefined> {
  return cache === undefined ? undefined : !(await cache.ready());
}

async function traceOutput(
  request: TraceOutputRequest,
): Promise<ProcessOutput> {
  const runtimeTrace = await readTrace(request.process.tracePath);
  return {
    ...(request.process.cache === undefined ||
    request.cacheAbsentBefore === undefined
      ? {}
      : {
          cacheAbsentBefore: request.cacheAbsentBefore,
          cacheCompletedBeforeModel:
            request.observation.cacheCompletedBeforeModel,
          cachePath: request.process.cache.path,
        }),
    exitCode: request.exitCode,
    runtimeTrace,
    stderr: request.stderr,
    stdout: request.observation.stdout,
    traceAbsent: runtimeTrace === "",
    traceCompletedBeforeModel: request.observation.traceCompletedBeforeModel,
  };
}

export { runObservedProcess, runTraceObservedProcess };
