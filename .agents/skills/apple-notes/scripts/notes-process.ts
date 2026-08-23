export class CommandFailure extends Error {
  constructor(readonly status: number) {
    super("");
  }
}

const decoder = new TextDecoder("utf-8", { fatal: true });

function decode(bytes: Uint8Array): string {
  try {
    return decoder.decode(bytes);
  } catch {
    throw new Error(`AppleScript returned invalid UTF-8`);
  }
}

export function runAppleScript(script: string): string {
  const result = Bun.spawnSync(["osascript"], {
    stdin: Buffer.from(script),
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    if (result.stdout.length > 0) process.stdout.write(result.stdout);
    if (result.stderr.length > 0) process.stderr.write(result.stderr);
    throw new CommandFailure(result.exitCode);
  }
  const stdout = decode(result.stdout);
  if (result.stderr.length > 0) process.stderr.write(result.stderr);
  return stdout.trimEnd();
}
