#!/usr/bin/env bun

const commandArgumentOffset = 2;
const unsupportedOptionExitCode = 2;

type CommandResult = Readonly<{
  status: number;
  stdout: string;
  stderr: string;
}>;

function run(command: string, arguments_: readonly string[]): CommandResult {
  try {
    const result = Bun.spawnSync([command, ...arguments_], {
      stderr: "pipe",
      stdout: "pipe",
    });
    return {
      status: result.exitCode,
      stderr: result.stderr.toString(),
      stdout: result.stdout.toString(),
    };
  } catch (error) {
    return {
      status: 127,
      stderr: `${command} is unavailable: ${String(error)}\n`,
      stdout: "",
    };
  }
}

function exitWithDetail(detail: string, status: number): never {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.exit(status);
}

function parseArguments(arguments_: readonly string[]): Readonly<{
  gitArguments: readonly string[];
  strict: boolean;
}> {
  let offset = 0;
  let strict = false;
  while (offset < arguments_.length) {
    if (arguments_[offset] === "--strict") {
      strict = true;
    } else {
      break;
    }
    offset += 1;
  }
  return {
    gitArguments: arguments_.slice(offset),
    strict,
  } as const;
}

function resolveGeneric(
  strict: boolean,
  gitArguments: readonly string[],
): string {
  const repository = run("git", [...gitArguments, "rev-parse", "--git-dir"]);
  if (repository.status !== 0) {
    if (strict) {
      exitWithDetail(repository.stderr, 1);
    }
    return "main";
  }
  for (const branch of ["main", "master", "trunk"] as const) {
    const result = run("git", [
      ...gitArguments,
      "show-ref",
      "-q",
      "--verify",
      `refs/heads/${branch}`,
    ]);
    if (result.status === 0) {
      return branch;
    }
    if (result.status !== 1) {
      if (strict) {
        exitWithDetail(result.stderr, result.status);
      }
      process.stderr.write(result.stderr);
    }
  }
  return "main";
}

function main(arguments_: readonly string[]): void {
  if (arguments_.includes("--bitbucket-cloud")) {
    process.stderr.write(
      "git-main-branch: unsupported option --bitbucket-cloud\n",
    );
    process.exit(unsupportedOptionExitCode);
  }
  const { strict, gitArguments } = parseArguments(arguments_);
  process.stdout.write(`${resolveGeneric(strict, gitArguments)}\n`);
}

if (import.meta.main) {
  main(process.argv.slice(commandArgumentOffset));
}

export { main };
