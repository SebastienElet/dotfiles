import type { ProcessOutput } from "./agent-memory-eval-process-types.ts";
import { diagnosticClass } from "./agent-memory-eval-diagnostics.ts";
import { superviseTimeout } from "./agent-memory-eval-supervision.ts";

type ProcessRequest = Readonly<{
  acceptedExitCodes?: readonly number[];
  command: readonly string[];
  cwd?: string;
  environment: Readonly<NodeJS.ProcessEnv>;
  stdin?: string;
  timeoutMilliseconds: number;
}>;

function runEvaluationProcess(
  ...[command, environment, timeoutMilliseconds]: readonly [
    readonly string[],
    Readonly<NodeJS.ProcessEnv>,
    number,
  ]
): Promise<ProcessOutput> {
  return runManagedProcess({ command, environment, timeoutMilliseconds });
}

async function runManagedProcess(
  request: ProcessRequest,
): Promise<ProcessOutput> {
  const child = Bun.spawn([...request.command], {
    ...(request.cwd === undefined ? {} : { cwd: request.cwd }),
    env: { ...process.env, ...request.environment },
    stderr: "pipe",
    stdin: request.stdin === undefined ? "ignore" : "pipe",
    stdout: "pipe",
  });
  const input = child.stdin;
  if (request.stdin !== undefined && input !== undefined) {
    await input.write(request.stdin);
    await input.end();
  }
  const guard = superviseTimeout(child, request.timeoutMilliseconds);
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]).finally(guard.clear);
  await guard.finish();
  assertProcessOutcome({
    exitCode,
    stderr,
    acceptedExitCodes: request.acceptedExitCodes,
    guard,
    timeoutMilliseconds: request.timeoutMilliseconds,
  });
  return { exitCode, stderr, stdout };
}

async function runManagedProcessToFile(
  ...[command, environment, destination, timeoutMilliseconds]: readonly [
    readonly string[],
    Readonly<NodeJS.ProcessEnv>,
    string,
    number,
  ]
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
  assertProcessOutcome({
    exitCode,
    stderr,
    acceptedExitCodes: [0],
    guard,
    timeoutMilliseconds,
  });
}

function assertProcessOutcome(
  request: Readonly<{
    acceptedExitCodes: readonly number[] | undefined;
    exitCode: number;
    guard: Readonly<{ expired: () => boolean }>;
    stderr: string;
    timeoutMilliseconds: number;
  }>,
): void {
  if (request.guard.expired()) {
    throw new Error(`process timed out after ${request.timeoutMilliseconds}ms`);
  }
  if (!(request.acceptedExitCodes ?? [0]).includes(request.exitCode)) {
    throw new Error(
      `process exit ${request.exitCode} (${diagnosticClass(request.stderr)})`,
    );
  }
}

export { runEvaluationProcess, runManagedProcess, runManagedProcessToFile };
export {
  runObservedProcess,
  runTraceObservedProcess,
} from "./agent-memory-eval-process-observation.ts";
export type {
  Agent,
  AgentCondition,
  ProcessOutput,
} from "./agent-memory-eval-process-types.ts";
