type Environment = Readonly<Record<string, string | undefined>>;

type BoundedCommandOptions = Readonly<{
  cwd: string;
  displayName?: string;
  environment: Environment;
  timeoutMilliseconds: number;
}>;

type CommandOutcome =
  | Readonly<{ kind: "exited"; status: number; stderr: string; stdout: string }>
  | Readonly<{ kind: "failed"; reason: string }>;

async function runBounded(
  name: string,
  arguments_: readonly string[],
  options: BoundedCommandOptions,
): Promise<CommandOutcome> {
  const displayName = options.displayName ?? name;
  const executable = Bun.which(name, { PATH: options.environment.PATH ?? "" });
  if (executable === null) {
    return { kind: "failed", reason: `${name} is not installed` };
  }
  const child = Bun.spawn([executable, ...arguments_], {
    cwd: options.cwd,
    env: { ...options.environment },
    stderr: "pipe",
    stdin: "ignore",
    stdout: "pipe",
  });
  let timedOut = false;
  const deadline = setTimeout(() => {
    timedOut = true;
    child.kill();
  }, options.timeoutMilliseconds);
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]).finally(() => {
    clearTimeout(deadline);
  });
  if (timedOut) {
    return {
      kind: "failed",
      reason: `${displayName} timed out after ${options.timeoutMilliseconds} ms`,
    };
  }
  if (child.signalCode !== null) {
    return {
      kind: "failed",
      reason: `${displayName} was terminated by ${child.signalCode}`,
    };
  }
  return { kind: "exited", status, stderr, stdout };
}

function commandFailure(name: string, status: number, stderr: string): string {
  const detail = stderr.trim().length > 0 ? stderr : `exit status ${status}`;
  return `${name} failed: ${detail}`;
}

export { commandFailure, runBounded };
export type { BoundedCommandOptions, CommandOutcome, Environment };
