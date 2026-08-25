const successfulExitCode = 0;

class CommandFailureError extends Error {
  public readonly status: number;

  public constructor(status: number) {
    super("");
    this.name = "CommandFailureError";
    this.status = status;
  }
}

function decodeOutput(output: readonly number[]): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      new Uint8Array(output),
    );
  } catch {
    throw new Error(`AppleScript returned invalid UTF-8`);
  }
}

const runAppleScript = (script: string): string => {
  const result = Bun.spawnSync(["osascript"], {
    stderr: "pipe",
    stdin: Buffer.from(script),
    stdout: "pipe",
  });
  if (result.exitCode !== successfulExitCode) {
    if (result.stdout.length > successfulExitCode) {
      process.stdout.write(result.stdout);
    }
    if (result.stderr.length > successfulExitCode) {
      process.stderr.write(result.stderr);
    }
    throw new CommandFailureError(result.exitCode);
  }
  const stdout = decodeOutput([...result.stdout]);
  if (result.stderr.length > successfulExitCode) {
    process.stderr.write(result.stderr);
  }
  return stdout.trimEnd();
};

export { CommandFailureError, runAppleScript };
