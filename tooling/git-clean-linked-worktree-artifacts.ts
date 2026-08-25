import { realpath } from "node:fs/promises";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1)]);
const worktreeFieldSchema = z
  .string()
  .startsWith("worktree ")
  .transform((field) => field.slice("worktree ".length))
  .pipe(z.string().min(1));

type CommandResult = Readonly<{
  status: number;
  stderr: string;
  stdout: Uint8Array;
}>;

function run(arguments_: readonly string[]): CommandResult {
  try {
    const result = Bun.spawnSync(["git", ...arguments_], {
      stderr: "pipe",
      stdout: "pipe",
    });
    return {
      status: result.exitCode,
      stderr: result.stderr.toString(),
      stdout: result.stdout,
    };
  } catch (error) {
    return {
      status: 127,
      stderr: `git is unavailable: ${String(error)}\n`,
      stdout: new Uint8Array(),
    };
  }
}

function fail(message: string, detail = ""): number {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.stderr.write(`git-clean-linked-worktree-artifacts: ${message}\n`);
  return 1;
}

function parseWorktreePaths(output: Uint8Array): readonly string[] {
  const worktreeOutput = z
    .string()
    .endsWith("\0\0")
    .parse(new TextDecoder("utf-8", { fatal: true }).decode(output));
  return worktreeOutput
    .slice(0, -2)
    .split("\0\0")
    .map((record) => {
      const fields = z.array(z.string()).min(2).parse(record.split("\0"));
      return worktreeFieldSchema.parse(fields[0]);
    });
}

async function isLinkedWorktree(worktree: string): Promise<boolean> {
  const worktreeList = run([
    "-C",
    worktree,
    "worktree",
    "list",
    "--porcelain",
    "-z",
  ]);
  if (worktreeList.status !== 0) {
    throw new Error(worktreeList.stderr.trim() || "Git worktree lookup failed");
  }
  const worktreePaths = parseWorktreePaths(worktreeList.stdout);
  const [resolvedWorktreePaths, resolvedWorktree] = await Promise.all([
    Promise.all(worktreePaths.map((candidate) => realpath(candidate))),
    realpath(worktree),
  ]);
  return resolvedWorktreePaths.indexOf(resolvedWorktree) > 0;
}

async function main(arguments_: readonly string[]): Promise<number> {
  const parsedArguments = argumentsSchema.safeParse(arguments_);
  if (!parsedArguments.success) {
    return fail("expected exactly one worktree path");
  }
  const [worktree] = parsedArguments.data;
  let linkedWorktree: boolean;
  try {
    linkedWorktree = await isLinkedWorktree(worktree);
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
  if (!linkedWorktree) {
    return fail("refusing to clean the primary or an unregistered worktree");
  }
  const cleanup = run(["-C", worktree, "clean", "-dfXq"]);
  if (cleanup.status !== 0) {
    return fail("ignored file cleanup failed", cleanup.stderr);
  }
  return 0;
}

export { main };
