class CheckCommandError extends Error {
  public readonly exitCode: number;

  public constructor(command: string, exitCode: number) {
    super(`${command} failed (${exitCode})`);
    this.name = "CheckCommandError";
    this.exitCode = exitCode;
  }
}

function checkCommand(
  command: readonly string[],
  options: Readonly<{ cwd?: string; env?: Readonly<NodeJS.ProcessEnv> }> = {},
): Bun.SyncSubprocess<"pipe", "pipe"> {
  const result = Bun.spawnSync([...command], {
    ...options,
    stdin: "ignore",
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new CheckCommandError(command.join(" "), result.exitCode);
  }
  return result;
}

function reportCheckFailure(error: unknown): void {
  process.stderr.write(
    `${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = error instanceof CheckCommandError ? error.exitCode : 1;
}

export { checkCommand, reportCheckFailure };
