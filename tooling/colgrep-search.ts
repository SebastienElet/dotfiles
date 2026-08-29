import { accessSync, constants, realpathSync } from "node:fs";
import {
  parseAndConfineResults,
  parseColgrepStatus,
  validateColgrepIndex,
} from "./colgrep-search-contract.ts";

const commandArgumentOffset = 2;

interface CommandResult {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
}

interface CommandExecution {
  readonly environment?: Readonly<Record<string, string | undefined>>;
  readonly run?: RunCommand;
}

type RunCommand = (
  binary: string,
  arguments_: readonly string[],
  environment?: Readonly<Record<string, string | undefined>>,
) => CommandResult;

function main(
  arguments_: readonly string[] = process.argv.slice(commandArgumentOffset),
): number {
  try {
    const [query] = arguments_;
    if (arguments_.length !== 1 || query === undefined || query.trim() === "") {
      throw new Error("exactly one non-empty conceptual query is required");
    }
    const git = requireExecutable(
      process.env.COLGREP_SEARCH_GIT_BIN ?? "git",
      "Git",
    );
    const colgrep = requireExecutable(
      process.env.COLGREP_SEARCH_COLGREP_BIN ?? "colgrep",
      "ColGrep",
    );
    const root = resolveCheckoutRoot(process.cwd(), git, runCommand);
    runRequired(colgrep, ["init", "-y", root]);
    const status = parseColgrepStatus(
      runRequired(colgrep, ["status", root]).stdout,
    );
    validateColgrepIndex(root, status);
    const search = runRequired(colgrep, [
      "search",
      "--json",
      "--no-update",
      query,
      root,
    ]);
    const results = parseAndConfineResults(search.stdout, root);
    process.stdout.write(`${JSON.stringify(results)}\n`);
    return 0;
  } catch (error) {
    process.stderr.write(
      `ColGrep search unavailable: ${message(error)}. Fall back to bounded rg/fd searches.\n`,
    );
    return 1;
  }
}

function resolveCheckoutRoot(
  cwd: string,
  git: string,
  run: RunCommand,
): string {
  const environment = gitEnvironment();
  const rootEvidence = runRequired(
    git,
    ["-C", cwd, "rev-parse", "--show-toplevel"],
    { environment, run },
  ).stdout;
  const root = canonicalPath(singleLine(rootEvidence, "Git checkout root"));
  const superproject = runRequired(
    git,
    ["-C", root, "rev-parse", "--show-superproject-working-tree"],
    { environment, run },
  ).stdout;
  if (superproject !== "") {
    throw new Error("the active repository is nested in a Git superproject");
  }
  return root;
}

function runRequired(
  binary: string,
  arguments_: readonly string[],
  execution: Readonly<CommandExecution> = {},
): CommandResult {
  const environment = execution.environment ?? process.env;
  const run = execution.run ?? runCommand;
  const result = run(binary, arguments_, environment);
  if (result.exitCode !== 0) {
    throw new Error(
      result.stderr.trimEnd() || `command failed with exit ${result.exitCode}`,
    );
  }
  return result;
}

function runCommand(
  binary: string,
  arguments_: readonly string[],
  environment: Readonly<Record<string, string | undefined>> = process.env,
): CommandResult {
  const result = Bun.spawnSync([binary, ...arguments_], {
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: decode(result.stderr, "stderr"),
    stdout: decode(result.stdout, "stdout"),
  };
}

function gitEnvironment(): Record<string, string | undefined> {
  return {
    ...process.env,
    GIT_COMMON_DIR: undefined,
    GIT_DIR: undefined,
    GIT_INDEX_FILE: undefined,
    GIT_PREFIX: undefined,
    GIT_WORK_TREE: undefined,
    LC_ALL: "C",
  };
}

function singleLine(output: string, label: string): string {
  const lines = output.endsWith("\n")
    ? output.slice(0, -1).split("\n")
    : output.split("\n");
  if (lines.length !== 1 || lines[0] === "") {
    throw new Error(`${label} is missing or ambiguous`);
  }
  const [line] = lines;
  if (line === undefined) {
    throw new Error(`${label} is missing or ambiguous`);
  }
  return line;
}

function canonicalPath(path: string): string {
  return realpathSync.native(path);
}

function requireExecutable(binary: string, name: string): string {
  const found = binary.includes("/") ? binary : Bun.which(binary);
  try {
    accessSync(found ?? binary, constants.X_OK);
  } catch {
    throw new Error(`${name} is required`);
  }
  return found ?? binary;
}

function decode(output: Readonly<ArrayLike<number>>, stream: string): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      Uint8Array.from(output),
    );
  } catch {
    throw new Error(`command ${stream} contains invalid UTF-8`);
  }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export { main, resolveCheckoutRoot };
