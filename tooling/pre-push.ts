import { z } from "zod";

const hookArgumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const lineFeed = "\n";
const objectIdSchema = z.string().regex(/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/u);
const refSchema = z.string().min(1).regex(/^\S+$/u);
const updateSchema = z
  .tuple([refSchema, objectIdSchema, refSchema, objectIdSchema])
  .readonly();

type CommandResult = Readonly<{
  status: number;
  stderr: string;
  stdout: string;
}>;
type CommandRunner = (
  command: string,
  arguments_: readonly string[],
) => CommandResult;

const run: CommandRunner = (command, arguments_) => {
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
};

function fail(message: string, detail = ""): number {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.stderr.write(`pre-push: ${message}\n`);
  return 1;
}

function parseUpdates(input: string): readonly z.output<typeof updateSchema>[] {
  const parsedInput = z.string().min(1).endsWith(lineFeed).parse(input);
  return parsedInput
    .slice(0, -lineFeed.length)
    .split(lineFeed)
    .map((line) => updateSchema.parse(line.split(" ")));
}

function requireOutput(
  options: Readonly<{
    runner: CommandRunner;
    command: string;
    arguments: readonly string[];
    failure: string;
  }>,
): string {
  const { runner, command, arguments: arguments_, failure } = options;
  const result = runner(command, arguments_);
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || failure);
  }
  return result.stdout;
}

function runStaticChecks(runner: CommandRunner): number {
  for (const command of [
    [
      "bun",
      ["--config=/dev/null", "--no-env-file", "tooling/lint-typescript.ts"],
    ],
    ["bun", ["--config=/dev/null", "--no-env-file", "run", "typecheck"]],
  ] as const) {
    const [commandName, commandArguments] = command;
    const result = runner(commandName, commandArguments);
    if (result.status !== 0) {
      return fail("static validation failed", result.stderr);
    }
  }
  return 0;
}

function main(
  arguments_: readonly string[],
  input: string,
  runner: CommandRunner = run,
): number {
  try {
    hookArgumentsSchema.parse(arguments_);
    const branchUpdates = parseUpdates(input).filter(([localRef]) =>
      localRef.startsWith("refs/heads/"),
    );
    if (branchUpdates.length === 0) {
      return 0;
    }
    const head = objectIdSchema.parse(
      requireOutput({
        runner,
        command: "git",
        arguments: ["rev-parse", "--verify", "HEAD"],
        failure: "HEAD lookup failed",
      }).trim(),
    );
    if (branchUpdates.some(([, localObjectId]) => localObjectId !== head)) {
      return fail("checkout every branch being pushed before validation");
    }
    const status = requireOutput({
      runner,
      command: "git",
      arguments: ["status", "--porcelain=v1", "--untracked-files=no"],
      failure: "worktree status failed",
    });
    if (status !== "") {
      return fail("commit or restore tracked worktree changes before pushing");
    }
    return runStaticChecks(runner);
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
}

export { main };
export type { CommandRunner };
