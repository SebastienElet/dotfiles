import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const integrationEnabled = process.env.COLGREP_INTEGRATION === "1";
const integrationTimeoutMilliseconds = 300_000;
const entryPoint = join(import.meta.dirname, "colgrep-search-cli.ts");
const fixtureRoots: string[] = [];
const mainSymbols = [
  "mainCheckoutTrackedSymbol",
  "mainCheckoutUntrackedSymbol",
] as const;
const activeSymbols = [
  "activeWorktreeTrackedSymbol",
  "activeWorktreeUntrackedSymbol",
] as const;
const neighborSymbols = [
  "neighborWorktreeOnlySymbol",
  "neighborWorktreeUntrackedSymbol",
] as const;

type SearchCase = readonly [string, string, string, readonly string[]];

setDefaultTimeout(integrationTimeoutMilliseconds);

afterEach(() => {
  for (const root of fixtureRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test.skipIf(!integrationEnabled)(
  "isolates tracked and untracked results across the main checkout and two worktrees",
  () => {
    const colgrep = requireExecutable(
      process.env.COLGREP_REAL_BIN ?? "colgrep",
    );
    const git = requireExecutable("git");
    const fixture = createDivergentWorktrees(git);
    const environment = {
      ...process.env,
      COLGREP_DATA_DIR: fixture.dataDirectory,
      COLGREP_SEARCH_COLGREP_BIN: colgrep,
      COLGREP_SEARCH_GIT_BIN: git,
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
    expectIsolatedSearches(fixture, environment);
  },
);

function createDivergentWorktrees(git: string): {
  readonly activeRoot: string;
  readonly dataDirectory: string;
  readonly homeDirectory: string;
  readonly mainRoot: string;
  readonly neighborRoot: string;
} {
  const root = mkdtempSync(join(tmpdir(), "colgrep-search-integration-"));
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
  divergeWorktrees({ activeRoot, git, mainRoot, neighborRoot });
  return {
    activeRoot,
    dataDirectory: join(root, "data"),
    homeDirectory: join(root, "home"),
    mainRoot,
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

function divergeWorktrees(options: {
  readonly activeRoot: string;
  readonly git: string;
  readonly mainRoot: string;
  readonly neighborRoot: string;
}): void {
  const { activeRoot, git, mainRoot, neighborRoot } = options;
  writeFileSync(
    join(mainRoot, "routing.ts"),
    `${readFileSync(join(mainRoot, "routing.ts"), "utf8")}export function mainCheckoutTrackedSymbol(): string { return "main tracked"; }\n`,
  );
  writeFileSync(
    join(mainRoot, "untracked.ts"),
    'export function mainCheckoutUntrackedSymbol(): string { return "main untracked"; }\n',
  );
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
  writeFileSync(
    join(neighborRoot, "untracked.ts"),
    'export function neighborWorktreeUntrackedSymbol(): string { return "neighbor untracked"; }\n',
  );
}

function expectIsolatedSearches(
  fixture: ReturnType<typeof createDivergentWorktrees>,
  environment: Readonly<NodeJS.ProcessEnv>,
): void {
  for (const [
    root,
    query,
    expectedSymbol,
    forbiddenSymbols,
  ] of isolatedSearchCases(fixture)) {
    const result = run([entryPoint, query], root, environment);
    expectSuccess(result);
    expect(result.stdout).toContain(expectedSymbol);
    for (const foreignSymbol of forbiddenSymbols) {
      expect(result.stdout).not.toContain(foreignSymbol);
    }
  }
}

function isolatedSearchCases(
  fixture: ReturnType<typeof createDivergentWorktrees>,
): readonly SearchCase[] {
  return [
    [
      fixture.mainRoot,
      "main checkout tracked symbol",
      mainSymbols[0],
      [...activeSymbols, ...neighborSymbols],
    ],
    [
      fixture.mainRoot,
      "main checkout untracked symbol",
      mainSymbols[1],
      [...activeSymbols, ...neighborSymbols],
    ],
    [
      fixture.activeRoot,
      "active worktree tracked symbol",
      activeSymbols[0],
      [...mainSymbols, ...neighborSymbols],
    ],
    [
      fixture.activeRoot,
      "active worktree untracked symbol",
      activeSymbols[1],
      [...mainSymbols, ...neighborSymbols],
    ],
    [
      fixture.neighborRoot,
      "neighbor worktree only symbol",
      neighborSymbols[0],
      [...mainSymbols, ...activeSymbols],
    ],
    [
      fixture.neighborRoot,
      "neighbor worktree untracked symbol",
      neighborSymbols[1],
      [...mainSymbols, ...activeSymbols],
    ],
  ] as const;
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
