import {
  chmod,
  cp,
  mkdir,
  mkdtemp,
  readdir,
  rm,
  symlink,
} from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const repositoryRoot = join(import.meta.dir, "..");
const temporaryRoots: string[] = [];
const executableMode = 0o755;
const realGit = Bun.which("git") ?? "git";

interface CommandResult {
  readonly status: number;
  readonly output: string;
  readonly stdout: string;
}
interface UpgradeResult extends CommandResult {
  readonly calls: string;
}
type Environment = Readonly<Record<string, string | undefined>>;
type FixtureGit = (...args: readonly string[]) => Promise<CommandResult>;
interface UpgradeFixture {
  readonly root: string;
  readonly home: string;
  readonly repository: string;
  readonly bin: string;
  readonly env: Environment;
  readonly git: FixtureGit;
  readonly run: (
    overrides?: Readonly<Record<string, string>>,
  ) => Promise<UpgradeResult>;
}

async function commandResult(
  command: readonly string[],
  cwd: string,
  env: Environment,
): Promise<CommandResult> {
  const child = Bun.spawn([...command], {
    cwd,
    env,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { status, output: stdout + stderr, stdout };
}

async function upgradeFixture(): Promise<UpgradeFixture> {
  const root = await mkdtemp(join(tmpdir(), "upgrade-"));
  temporaryRoots.push(root);
  const home = join(root, "home");
  const repository = join(home, ".dotfiles");
  const bin = join(root, "bin");
  await prepareFixtureFiles(repository, home, bin);
  const env = {
    HOME: home,
    PATH: bin,
    REAL_GIT: realGit,
    UPGRADE_LOG: join(root, "calls"),
    GIT_CONFIG_NOSYSTEM: "1",
    GIT_CONFIG_GLOBAL: "/dev/null",
    GIT_TERMINAL_PROMPT: "0",
  };
  const git: FixtureGit = (
    ...args: readonly string[]
  ): Promise<CommandResult> =>
    commandResult([realGit, "-C", repository, ...args], root, env);
  await initializeRepository(git, root);
  return {
    root,
    home,
    repository,
    bin,
    env,
    git,
    async run(
      overrides: Readonly<Record<string, string>> = {},
    ): Promise<UpgradeResult> {
      await Bun.write(env.UPGRADE_LOG, "");
      const result = await commandResult(
        [join(repository, "tooling/upgrade")],
        root,
        { ...env, ...overrides },
      );
      return { ...result, calls: await Bun.file(env.UPGRADE_LOG).text() };
    },
  };
}

async function prepareFixtureFiles(
  repository: string,
  home: string,
  bin: string,
): Promise<void> {
  await Promise.all(
    [
      join(repository, "tooling"),
      join(repository, "home/.config/nvim"),
      join(home, ".moon/bin"),
      bin,
    ].map((directory) => mkdir(directory, { recursive: true })),
  );
  await copyUpgradeFiles(repository);
  await Bun.write(
    join(repository, "package.json"),
    '{"volta":{"node":"24.18.1"}}',
  );
  await Bun.write(
    join(home, "pinned-package.json"),
    '{"volta":{"node":"24.19.0"}}',
  );
  await Bun.write(
    join(repository, "home/.config/nvim/lazy-lock.json"),
    "base\n",
  );
  await symlink(
    join(repositoryRoot, "node_modules"),
    join(repository, "node_modules"),
  );
  await symlink(process.execPath, join(bin, "bun"));
  await installBoundaryCommands(bin);
  await installFakeCommands(bin, home);
}

async function installBoundaryCommands(bin: string): Promise<void> {
  for (const command of ["bash", "dirname"]) {
    const executable = Bun.which(command);
    if (executable === null) {
      throw new Error(`${command} missing from test environment`);
    }
    await symlink(executable, join(bin, command));
  }
}

async function copyUpgradeFiles(repository: string): Promise<void> {
  const names = await readdir(import.meta.dir);
  const files = names.filter(
    (name) =>
      name === "upgrade" ||
      name === "git-main-branch" ||
      name === "node-version-contract.ts" ||
      ((name.startsWith("upgrade-") || name.startsWith("git-main-branch-")) &&
        name.endsWith(".ts")),
  );
  for (const file of files) {
    await cp(join(import.meta.dir, file), join(repository, "tooling", file));
  }
}

async function installFakeCommands(bin: string, home: string): Promise<void> {
  const source = await Bun.file(
    join(import.meta.dir, "upgrade-test-command.ts"),
  ).text();
  for (const command of [
    "git",
    "brew",
    "mas",
    "volta",
    "npm",
    "claude",
    "codex",
    "nvim",
    "date",
  ]) {
    const target = join(bin, command);
    await Bun.write(target, `#!${process.execPath}\n${source}`);
    await chmod(target, executableMode);
  }
  await symlink(
    join(repositoryRoot, "node_modules"),
    join(bin, "node_modules"),
  );
  const moon = join(home, ".moon/bin/moon");
  await Bun.write(moon, `#!${process.execPath}\n${source}`);
  await chmod(moon, executableMode);
  await symlink(
    join(repositoryRoot, "node_modules"),
    join(home, "node_modules"),
  );
}

async function initializeRepository(
  git: FixtureGit,
  root: string,
): Promise<void> {
  const remote = join(root, "remote.git");
  for (const args of [
    ["init", "--bare", remote],
    ["init", "-b", "main"],
    ["config", "user.email", "upgrade@example.test"],
    ["config", "user.name", "Upgrade test"],
    ["config", "core.hooksPath", "/dev/null"],
    ["add", "."],
    ["commit", "-qm", "base"],
    ["remote", "add", "origin", remote],
    ["push", "-qu", "origin", "main"],
  ]) {
    const result = await git(...args);
    if (result.status !== 0) {
      throw new Error(result.output);
    }
  }
}

async function cleanupUpgradeFixtures(): Promise<void> {
  for (const root of temporaryRoots.splice(0)) {
    await rm(root, { recursive: true, force: true });
  }
}

export { commandResult, upgradeFixture, cleanupUpgradeFixtures };
