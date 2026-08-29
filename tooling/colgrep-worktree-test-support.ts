import {
  chmodSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { afterEach } from "bun:test";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

interface CommandResult {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
}

interface LinkedWorktreeFixture {
  readonly activeResult: { readonly unit: { readonly file: string } };
  readonly environment: Readonly<Record<string, string>>;
  readonly indexDirectory: string;
  readonly invocations: string;
  readonly linkedRoot: string;
  readonly mainRoot: string;
  readonly root: string;
}

interface FixtureOptions {
  readonly mode?: string;
}

interface RepositoryFixture {
  readonly linkedRoot: string;
  readonly mainRoot: string;
  readonly root: string;
}

const executableFileMode = 0o755;
const invocationsSchema = z.array(z.string());
const project = import.meta.dirname;
const entryPoint = join(project, "colgrep-worktree");
const provider = join(project, "colgrep-worktree-test-provider");
const gitProvider = join(project, "colgrep-worktree-git-test-provider");
const fixtureRoots: string[] = [];

afterEach(() => {
  for (const root of fixtureRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function createLinkedWorktreeFixture(
  options: FixtureOptions = {},
): LinkedWorktreeFixture {
  chmodSync(provider, executableFileMode);
  chmodSync(gitProvider, executableFileMode);
  const repository = createRepositoryFixture();
  return createColgrepFixture(repository, options);
}

function createRepositoryFixture(): RepositoryFixture {
  const root = mkdtempSync(join(tmpdir(), "colgrep-worktree-test-"));
  fixtureRoots.push(root);
  const mainRoot = join(root, "main");
  const linkedRoot = join(root, "linked");
  initializeMainRepository(root, mainRoot);
  run(
    [
      "git",
      "-C",
      mainRoot,
      "worktree",
      "add",
      "-q",
      "-b",
      "feature",
      linkedRoot,
    ],
    root,
  );
  return { linkedRoot, mainRoot, root };
}

function initializeMainRepository(root: string, mainRoot: string): void {
  run(["git", "init", "-q", "-b", "main", mainRoot], root);
  writeFileSync(
    join(mainRoot, "tracked.ts"),
    "export const mainSymbol = true;\n",
  );
  run(["git", "-C", mainRoot, "add", "tracked.ts"], root);
  run(
    [
      "git",
      "-C",
      mainRoot,
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.test",
      "commit",
      "-qm",
      "fixture",
    ],
    root,
  );
}

function createColgrepFixture(
  repository: RepositoryFixture,
  options: FixtureOptions,
): LinkedWorktreeFixture {
  const { linkedRoot, mainRoot, root } = repository;
  const canonicalLinkedRoot = realpathSync.native(linkedRoot);
  const gitDirectory = run(
    ["git", "-C", canonicalLinkedRoot, "rev-parse", "--absolute-git-dir"],
    root,
  ).stdout.trimEnd();
  const indexDirectory = join(root, "data", "indices", "linked-fixture");
  const invocations = join(root, "invocations.jsonl");
  const activeResult = {
    unit: { file: join(canonicalLinkedRoot, "tracked.ts") },
  };
  return {
    activeResult,
    environment: {
      ...stringEnvironment(process.env),
      COLGREP_TEST_INDEX_DIRECTORY: indexDirectory,
      COLGREP_TEST_FOREIGN_ROOT: realpathSync.native(mainRoot),
      COLGREP_TEST_GIT_COMMON_DIRECTORY: realpathSync.native(
        join(mainRoot, ".git"),
      ),
      COLGREP_TEST_GIT_DIRECTORY: realpathSync.native(gitDirectory),
      COLGREP_TEST_INVOCATIONS: invocations,
      COLGREP_TEST_MODE: options.mode ?? "healthy",
      COLGREP_TEST_PROJECT_ROOT: canonicalLinkedRoot,
      COLGREP_TEST_RESULTS: JSON.stringify([activeResult]),
      COLGREP_WORKTREE_COLGREP_BIN: provider,
      COLGREP_WORKTREE_GIT_BIN: requireExecutable("git"),
    },
    indexDirectory,
    invocations,
    linkedRoot: canonicalLinkedRoot,
    mainRoot: realpathSync.native(mainRoot),
    root,
  };
}

function readInvocations(fixture: LinkedWorktreeFixture): readonly string[][] {
  if (!existsSync(fixture.invocations)) {
    return [];
  }
  return readFileSync(fixture.invocations, "utf8")
    .trimEnd()
    .split("\n")
    .filter(Boolean)
    .map((line) => invocationsSchema.parse(JSON.parse(line)));
}

function runEntryPoint(
  cwd: string,
  query: string,
  environment: Readonly<Record<string, string>>,
): CommandResult {
  return run([entryPoint, query], cwd, environment);
}

function run(
  command: readonly string[],
  cwd: string,
  environment: Readonly<Record<string, string>> = stringEnvironment(
    process.env,
  ),
): CommandResult {
  const result = Bun.spawnSync([...command], {
    cwd,
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function requireExecutable(name: string): string {
  const executable = Bun.which(name);
  if (executable === null) {
    throw new Error(`${name} is required`);
  }
  return executable;
}

function stringEnvironment(
  environment: Readonly<NodeJS.ProcessEnv>,
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(environment).filter(
      (
        entry: readonly [string, string | undefined],
      ): entry is [string, string] => entry[1] !== undefined,
    ),
  );
}

export {
  createLinkedWorktreeFixture,
  gitProvider,
  readInvocations,
  runEntryPoint,
  type LinkedWorktreeFixture,
};
