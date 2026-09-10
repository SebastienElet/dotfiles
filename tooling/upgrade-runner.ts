type State = "succeeded" | "failed" | "not-attempted" | "unsupported";
type Result<Value> =
  | Readonly<{ state: "succeeded"; value: Value }>
  | Readonly<{ state: "failed" | "not-attempted" }>;
type Report = (state: State, label: string, detail?: string) => void;
type CommandOptions = Readonly<{
  cwd?: string;
  env?: Readonly<Record<string, string>>;
  capture?: boolean;
  required?: boolean;
}>;
type UpgradeRunner = Readonly<{
  command: (
    label: string,
    command: readonly string[],
    options?: CommandOptions,
  ) => Promise<Result<string>>;
  attempt: <Value>(
    label: string,
    action: () => Value | Promise<Value>,
  ) => Promise<Result<Value>>;
  skip: (label: string, reason: string) => void;
  unsupported: (label: string, reason: string) => void;
  finish: () => number;
}>;

function runCommand(
  report: Report,
  label: string,
  options: CommandOptions & Readonly<{ command: readonly string[] }>,
): Promise<Result<string>> {
  const { command } = options;
  const [executable] = command;
  if (executable === undefined || Bun.which(executable) === null) {
    const state = options.required === true ? "failed" : "not-attempted";
    report(state, label, `${executable ?? "command"} not found`);
    return Promise.resolve({ state });
  }
  return attempt(report, label, async () => {
    const child = Bun.spawn([...command], {
      cwd: options.cwd ?? process.cwd(),
      env: { ...process.env, ...options.env },
      stdin: "inherit",
      stdout: options.capture === true ? "pipe" : "inherit",
      stderr: "inherit",
    });
    const [status, output] = await Promise.all([
      child.exited,
      options.capture === true
        ? new Response(child.stdout).text()
        : Promise.resolve(""),
    ]);
    if (status !== 0) {
      throw new Error(`command exited ${status}`);
    }
    return output;
  });
}

async function attempt<Value>(
  report: Report,
  label: string,
  action: () => Value | Promise<Value>,
): Promise<Result<Value>> {
  try {
    const value = await action();
    report("succeeded", label);
    return { state: "succeeded", value };
  } catch (error) {
    report(
      "failed",
      label,
      error instanceof Error ? error.message : String(error),
    );
    return { state: "failed" };
  }
}

function createUpgradeRunner(): UpgradeRunner {
  let failures = 0;
  const report: Report = (state, label, detail) => {
    if (state === "failed") {
      failures += 1;
    }
    process.stdout.write(
      `  [${state}] ${label}${detail === undefined ? "" : `: ${detail}`}\n`,
    );
  };
  return {
    command: (label, command, options = {}) =>
      runCommand(report, label, { ...options, command }),
    attempt: (label, action) => attempt(report, label, action),
    skip: (label, reason) => {
      report("not-attempted", label, reason);
    },
    unsupported: (label, reason) => {
      report("unsupported", label, reason);
    },
    finish(): number {
      if (failures > 0) {
        process.stderr.write(
          `Upgrade finished with ${failures} failed operations; earlier or partial effects remain.\n`,
        );
      }
      return failures > 0 ? 1 : 0;
    },
  };
}

export { createUpgradeRunner, type UpgradeRunner };
