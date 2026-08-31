import { chmod, cp, mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { tmpdir } from "node:os";

const repositoryRoot = fileURLToPath(new URL("..", import.meta.url));
const temporaryDirectories: string[] = [];

interface CommandResult {
  readonly calls: string;
  readonly finalPackageJson?: unknown;
  readonly output: string;
  readonly status: number;
}

interface UpgradeScenario {
  readonly failVoltaCommand?: "install" | "pin";
  readonly includeBun?: boolean;
  readonly includeDependencies?: boolean;
  readonly includeVolta?: boolean;
  readonly pinnedPackage: unknown;
}

const executableMode = 0o755;

async function createTemporaryDirectory(prefix: string): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), prefix));
  temporaryDirectories.push(directory);
  return directory;
}

async function writeExecutable(path: string, body: string): Promise<void> {
  await Bun.write(path, `#!/bin/sh\n${body}\n`);
  await chmod(path, executableMode);
}

async function createContractFixture(
  root: string,
  packageJson: unknown,
  directoryName = "dotfiles",
): Promise<string> {
  const fixture = join(root, directoryName);
  await mkdir(join(fixture, "tooling"), { recursive: true });
  await cp(
    join(repositoryRoot, "tooling/node-version-contract.ts"),
    join(fixture, "tooling/node-version-contract.ts"),
  );
  await cp(join(repositoryRoot, "bun.lock"), join(fixture, "bun.lock"));
  await Bun.write(join(fixture, "package.json"), JSON.stringify(packageJson));
  await symlink(
    join(repositoryRoot, "node_modules"),
    join(fixture, "node_modules"),
  );
  return fixture;
}

async function run(
  command: readonly string[],
  environment: Readonly<Record<string, string>>,
): Promise<Omit<CommandResult, "calls">> {
  const child = Bun.spawn([...command], {
    cwd: repositoryRoot,
    env: { ...process.env, ...environment },
    stdout: "pipe",
    stderr: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { status, output: `${stdout}${stderr}` };
}

async function runMakeNode(
  packageJson: unknown,
  includeDependencies = true,
): Promise<CommandResult> {
  const root = await createTemporaryDirectory("node-make-");
  const fixture = await createContractFixture(root, packageJson);
  const bin = join(root, "bin");
  const home = join(root, "home");
  const voltaBin = join(home, ".volta/bin");
  const commandLog = join(root, "volta.log");
  if (!includeDependencies) {
    await rm(join(fixture, "node_modules"));
  }
  await mkdir(bin, { recursive: true });
  await mkdir(voltaBin, { recursive: true });
  await writeExecutable(join(bin, "brew"), "exit 0");
  await writeExecutable(
    join(bin, "volta"),
    String.raw`printf "%s\n" "$*" >>"$COMMAND_LOG"`,
  );
  await writeExecutable(
    join(bin, "bun"),
    'if [ "$1" = --config=/dev/null ]; then exit 0; fi\nexec "$REAL_BUN" "$@"',
  );
  const result = await run(
    [
      "make",
      "-f",
      join(repositoryRoot, "Makefile"),
      "node",
      `HOME=${home}`,
      `BREW_BIN=${bin}`,
      `VOLTA_BIN=${voltaBin}`,
      `DOTFILES_PATH=${fixture}`,
    ],
    {
      COMMAND_LOG: commandLog,
      PATH: `${bin}:/usr/bin:/bin`,
      REAL_BUN: process.execPath,
    },
  );
  return {
    ...result,
    calls: await Bun.file(commandLog)
      .text()
      .catch(() => ""),
  };
}

async function createUpgradeFakes(
  root: string,
  scenario: UpgradeScenario,
): Promise<string> {
  const bin = join(root, "bin");
  const moonBin = join(root, "home", ".moon/bin");
  await mkdir(bin, { recursive: true });
  await mkdir(moonBin, { recursive: true });
  for (const command of ["brew", "mas", "npm"]) {
    await writeExecutable(join(bin, command), "exit 0");
  }
  await writeExecutable(
    join(bin, "date"),
    String.raw`printf '%s\n' 2026-08-23T00:00:00Z`,
  );
  await writeExecutable(join(moonBin, "moon"), "exit 0");
  if (scenario.includeVolta !== false) {
    await writeExecutable(
      join(bin, "volta"),
      String.raw`printf "%s\n" "$*" >>"$VOLTA_LOG"
if [ "$FAIL_VOLTA_COMMAND" = "$1" ]; then printf "%s\n" "simulated $1 failure" >&2; exit 23; fi
if [ "$1" = pin ]; then cp "$PINNED_PACKAGE" package.json; fi`,
    );
  }
  if (scenario.includeBun !== false) {
    await symlink(process.execPath, join(bin, "bun"));
  }
  return bin;
}

async function runUpgrade(scenario: UpgradeScenario): Promise<CommandResult> {
  const root = await createTemporaryDirectory("node-upgrade-");
  const home = join(root, "home");
  const fixture = await createContractFixture(
    home,
    { volta: { node: "24.18.1" } },
    ".dotfiles",
  );
  if (scenario.includeDependencies === false) {
    await rm(join(fixture, "node_modules"));
  }
  const pinnedPackagePath = join(root, "pinned-package.json");
  const voltaLog = join(root, "volta.log");
  await Bun.write(pinnedPackagePath, JSON.stringify(scenario.pinnedPackage));
  const bin = await createUpgradeFakes(root, scenario);
  const result = await run([join(repositoryRoot, "tooling/upgrade")], {
    FAIL_VOLTA_COMMAND: scenario.failVoltaCommand ?? "",
    HOME: home,
    PATH: `${bin}:/usr/bin:/bin`,
    PINNED_PACKAGE: pinnedPackagePath,
    VOLTA_LOG: voltaLog,
  });
  return {
    ...result,
    calls: await Bun.file(voltaLog)
      .text()
      .catch(() => ""),
    finalPackageJson: await Bun.file(join(fixture, "package.json")).json(),
  };
}

async function cleanupNodeVersionFixtures(): Promise<void> {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true })),
  );
}

export {
  cleanupNodeVersionFixtures,
  runMakeNode,
  runUpgrade,
  type CommandResult,
  type UpgradeScenario,
};
