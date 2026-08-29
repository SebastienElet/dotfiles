import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const integrationEnabled = process.env.COLGREP_INTEGRATION === "1";
const integrationTimeoutMilliseconds = 300_000;
const entryPoint = join(import.meta.dirname, "colgrep-worktree");
const fixtureRoots: string[] = [];

setDefaultTimeout(integrationTimeoutMilliseconds);

afterEach(() => {
  for (const root of fixtureRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test.skipIf(!integrationEnabled)(
  "keeps tracked and untracked results inside one of two divergent worktrees",
  () => {
    const colgrep = requireExecutable(
      process.env.COLGREP_REAL_BIN ?? "colgrep",
    );
    const git = requireExecutable("git");
    const fixture = createDivergentWorktrees(git);
    const environment = {
      ...process.env,
      COLGREP_DATA_DIR: fixture.dataDirectory,
      COLGREP_WORKTREE_COLGREP_BIN: colgrep,
      COLGREP_WORKTREE_GIT_BIN: git,
      HF_HOME: join(fixture.homeDirectory, ".cache", "huggingface"),
      HOME: fixture.homeDirectory,
      XDG_CACHE_HOME: join(fixture.homeDirectory, ".cache"),
      XDG_CONFIG_HOME: join(fixture.homeDirectory, ".config"),
    };

    expectSuccess(
      run(
        [colgrep, "init", "-y", fixture.neighborRoot],
        fixture.neighborRoot,
        environment,
      ),
    );
    const tracked = run(
      [entryPoint, "active worktree tracked symbol"],
      fixture.activeRoot,
      environment,
    );
    const untracked = run(
      [entryPoint, "active worktree untracked symbol"],
      fixture.activeRoot,
      environment,
    );

    expectSuccess(tracked);
    expectSuccess(untracked);
    expect(tracked.stdout).toContain("activeWorktreeTrackedSymbol");
    expect(untracked.stdout).toContain("activeWorktreeUntrackedSymbol");
    expect(`${tracked.stdout}${untracked.stdout}`).not.toContain(
      "neighborWorktreeOnlySymbol",
    );
  },
);

function createDivergentWorktrees(git: string): {
  readonly activeRoot: string;
  readonly dataDirectory: string;
  readonly homeDirectory: string;
  readonly neighborRoot: string;
} {
  const root = mkdtempSync(join(tmpdir(), "colgrep-worktree-integration-"));
  fixtureRoots.push(root);
  const mainRoot = join(root, "main");
  const activeRoot = join(root, "active");
  const neighborRoot = join(root, "neighbor");
  expectSuccess(run([git, "init", "-q", "-b", "main", mainRoot], root));
  writeFileSync(
    join(mainRoot, "routing.ts"),
    "export const baselineSymbol = true;\n",
  );
  commitAll(git, mainRoot, "baseline");
  addLinkedWorktree({
    branch: "active",
    git,
    mainRoot,
    root,
    worktreeRoot: activeRoot,
  });
  addLinkedWorktree({
    branch: "neighbor",
    git,
    mainRoot,
    root,
    worktreeRoot: neighborRoot,
  });
  divergeWorktrees(git, activeRoot, neighborRoot);
  return {
    activeRoot,
    dataDirectory: join(root, "data"),
    homeDirectory: join(root, "home"),
    neighborRoot,
  };
}

function addLinkedWorktree(options: {
  readonly branch: string;
  readonly git: string;
  readonly mainRoot: string;
  readonly root: string;
  readonly worktreeRoot: string;
}): void {
  const { branch, git, mainRoot, root, worktreeRoot } = options;
  expectSuccess(
    run(
      [
        git,
        "-C",
        mainRoot,
        "worktree",
        "add",
        "-q",
        "-b",
        branch,
        worktreeRoot,
      ],
      root,
    ),
  );
}

function divergeWorktrees(
  git: string,
  activeRoot: string,
  neighborRoot: string,
): void {
  writeFileSync(
    join(activeRoot, "routing.ts"),
    `${readFileSync(join(activeRoot, "routing.ts"), "utf8")}export function activeWorktreeTrackedSymbol(): string { return "active tracked"; }\n`,
  );
  writeFileSync(
    join(activeRoot, "untracked.ts"),
    'export function activeWorktreeUntrackedSymbol(): string { return "active untracked"; }\n',
  );
  writeFileSync(
    join(neighborRoot, "routing.ts"),
    `${readFileSync(join(neighborRoot, "routing.ts"), "utf8")}export function neighborWorktreeOnlySymbol(): string { return "neighbor"; }\n`,
  );
  commitAll(git, neighborRoot, "neighbor divergence");
}

function commitAll(git: string, repository: string, message: string): void {
  expectSuccess(run([git, "-C", repository, "add", "."], repository));
  expectSuccess(
    run(
      [
        git,
        "-C",
        repository,
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.test",
        "commit",
        "-qm",
        message,
      ],
      repository,
    ),
  );
}

function run(
  command: readonly string[],
  cwd: string,
  environment: Readonly<NodeJS.ProcessEnv> = process.env,
): {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
} {
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

function expectSuccess(result: ReturnType<typeof run>): void {
  if (result.exitCode !== 0) {
    throw new Error(`command failed (${result.exitCode}): ${result.stderr}`);
  }
}

function requireExecutable(name: string): string {
  const executable = name.includes("/") ? name : Bun.which(name);
  if (executable === null) {
    throw new Error(`${name} is required`);
  }
  return executable;
}
