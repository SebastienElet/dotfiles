import { targetsFromMakefileDiff } from "./ci-smoke-targets-makefile.ts";

const decoder = new TextDecoder("utf-8", { fatal: true });
const infrastructurePaths = new Set([
  "install.sh",
  ".github/workflows/test.yml",
  "tooling/ci-smoke-targets",
  "tooling/ci-smoke-targets-makefile.ts",
  "tooling/ci-smoke-targets.ts",
]);

type GitResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: Uint8Array;
}>;

class SelectorError extends Error {
  constructor(
    message: string,
    readonly details = "",
  ) {
    super(message);
  }
}

class GitError extends Error {
  constructor(readonly details: string) {
    super("Git command failed");
  }
}

function runGit(arguments_: readonly string[]): GitResult {
  const result = Bun.spawnSync({
    cmd: ["git", ...arguments_],
    stderr: "pipe",
    stdout: "pipe",
  });

  return {
    exitCode: result.exitCode,
    stderr: new TextDecoder().decode(result.stderr),
    stdout: result.stdout,
  };
}

function requireGit(arguments_: readonly string[]): Uint8Array {
  const result = runGit(arguments_);
  if (result.exitCode !== 0) {
    throw new GitError(result.stderr);
  }
  return result.stdout;
}

function verifyCommit(reference: string, role: "base" | "head"): void {
  const result = runGit(["rev-parse", "--verify", `${reference}^{commit}`]);
  if (result.exitCode !== 0) {
    throw new SelectorError(
      `invalid ${role} commit: ${reference}`,
      result.stderr,
    );
  }
}

function decodeGitEvidence(bytes: Uint8Array): string {
  try {
    return decoder.decode(bytes);
  } catch {
    throw new SelectorError("Git returned non-UTF-8 evidence");
  }
}

function makefileAt(reference: string): string | undefined {
  const result = runGit(["show", `${reference}:Makefile`]);
  if (result.exitCode !== 0) {
    process.stderr.write(result.stderr);
    return undefined;
  }
  return decodeGitEvidence(result.stdout);
}

function collectMakefileTargets(base: string, head: string): readonly string[] {
  const oldContents = makefileAt(base);
  const newContents = makefileAt(head);
  if (oldContents === undefined || newContents === undefined) {
    return ["all"];
  }

  const diff = decodeGitEvidence(
    requireGit([
      "diff",
      "--no-ext-diff",
      "--unified=0",
      base,
      head,
      "--",
      "Makefile",
    ]),
  );
  try {
    return targetsFromMakefileDiff(oldContents, newContents, diff);
  } catch (error) {
    if (error instanceof Error) {
      throw new SelectorError(error.message);
    }
    throw error;
  }
}

function changedPaths(base: string, head: string): readonly string[] {
  const output = requireGit([
    "diff",
    "--name-only",
    "--no-renames",
    "-z",
    `${base}...${head}`,
  ]);
  if (output.length === 0) {
    throw new SelectorError("the commit range contains no changes");
  }
  return decodeGitEvidence(output).split("\0").filter(Boolean);
}

export function selectSmokeTargets(
  base: string,
  head: string,
): readonly string[] {
  verifyCommit(base, "base");
  verifyCommit(head, "head");

  const paths = changedPaths(base, head);
  if (paths.some((path) => infrastructurePaths.has(path))) {
    return ["all"];
  }
  const candidates = paths.includes("Makefile")
    ? collectMakefileTargets(base, head)
    : [];
  if (candidates.includes("all")) {
    return ["all"];
  }
  return [...new Set(candidates)].sort();
}

function report(error: unknown): void {
  if (error instanceof SelectorError) {
    process.stderr.write(error.details);
    process.stderr.write(`ci-smoke-targets: ${error.message}\n`);
    return;
  }
  if (error instanceof GitError) {
    process.stderr.write(error.details);
    return;
  }
  process.stderr.write(`ci-smoke-targets: ${String(error)}\n`);
}

export function main(arguments_: readonly string[]): number {
  try {
    if (arguments_.length !== 2) {
      throw new SelectorError("usage: ci-smoke-targets BASE HEAD");
    }
    process.stdout.write(
      `${JSON.stringify(selectSmokeTargets(arguments_[0]!, arguments_[1]!))}\n`,
    );
    return 0;
  } catch (error) {
    report(error);
    return 1;
  }
}
