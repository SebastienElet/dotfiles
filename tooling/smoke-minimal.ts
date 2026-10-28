import { CheckCommandError, reportCheckFailure } from "./check-command.ts";
import { accessSync, constants, lstatSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { createHash } from "node:crypto";
import { z } from "zod";

const pathSchema = z.string().min(1);
const snapshotExclusion = ".claude/plugins/cache/*/*/*/.in_use";
const excludedSnapshotPath = new Bun.Glob(snapshotExclusion);
const isExcludedSnapshotPath =
  excludedSnapshotPath.match.bind(excludedSnapshotPath);
const snapshotPaths = [
  ".agents/skills",
  ".arnes.yaml",
  ".claude",
  ".codex/AGENTS.md",
  ".codex/agents",
  ".config/bat",
  ".config/cspell",
  ".config/fish",
  ".config/git/config.delta",
  ".config/git/ignore",
  ".config/nvim",
  ".config/starship.toml",
  ".config/tmux",
  ".config/wezterm",
  ".gitconfig",
  ".local/bin/agent-handoff",
  ".local/bin/arnes",
  ".local/bin/claude",
  ".tmux/plugins/tpm",
  ".volta/bin/codex",
  ".volta/bin/node",
  ".volta/bin/pnpm",
  "Library/Spelling",
  "cspell.json",
];
const installCommand = [
  "moon",
  "exec",
  "--quiet",
  "--ignore-ci-checks",
  "repository:install",
];

function commandFailure(
  command: readonly string[],
  stdout: string,
  stderr: string,
): void {
  process.stderr.write(
    `Command failed: ${JSON.stringify(command)}\nstdout:\n${stdout}\nstderr:\n${stderr}\n`,
  );
}

function checkSmokeCommand(
  command: readonly string[],
  options: Readonly<{ cwd?: string; env?: Readonly<NodeJS.ProcessEnv> }> = {},
): Bun.SyncSubprocess<"pipe", "pipe"> {
  try {
    const result = Bun.spawnSync([...command], {
      ...options,
      stdin: "ignore",
      stdout: "pipe",
      stderr: "pipe",
    });
    if (result.exitCode !== 0) {
      commandFailure(
        command,
        result.stdout.toString(),
        result.stderr.toString(),
      );
      throw new CheckCommandError(command.join(" "), result.exitCode);
    }
    return result;
  } catch (error) {
    if (error instanceof CheckCommandError) {
      throw error;
    }
    commandFailure(command, "", String(error));
    throw error;
  }
}

function verifyInstallation(home: string, root: string): void {
  checkSmokeCommand([
    "brew",
    "bundle",
    "check",
    "--quiet",
    "--no-upgrade",
    "--file",
    join(root, "Brewfile"),
  ]);
  const executables = [
    ...["agent-handoff", "arnes", "claude"].map((command) =>
      join(home, ".local/bin", command),
    ),
    ...["codex", "node", "pnpm"].map((command) =>
      join(home, ".volta/bin", command),
    ),
  ];
  for (const executable of executables) {
    accessSync(executable, constants.X_OK);
  }
  checkSmokeCommand(
    [
      "bun",
      "--config=/dev/null",
      "--no-env-file",
      "--no-install",
      join(root, "tooling/node-version-contract.ts"),
      "verify-runtime",
      join(root, "package.json"),
    ],
    {
      cwd: "/",
      env: {
        ...process.env,
        VOLTA_HOME: join(home, ".volta"),
        PATH: `${join(home, ".volta/bin")}:${process.env.PATH ?? ""}`,
      },
    },
  );
}

function archiveDigest(
  home: string,
  paths: readonly string[],
  options: readonly string[],
): string {
  const result = checkSmokeCommand(["tar", ...options, "-cf", "-", ...paths], {
    cwd: home,
  });
  return createHash("sha256").update(result.stdout).digest("hex");
}

type HomeSnapshot = Readonly<{
  digest: string;
  entries: Readonly<Record<string, string>>;
}>;

function observedPaths(home: string, path: string): string[] {
  if (isExcludedSnapshotPath(path)) {
    return [];
  }
  const absolute = join(home, path);
  const metadata = lstatSync(absolute, { throwIfNoEntry: false });
  if (metadata === undefined) {
    return [];
  }
  const children = metadata.isDirectory()
    ? readdirSync(absolute)
        .toSorted()
        .flatMap((name) => observedPaths(home, join(path, name)))
    : [];
  return [path, ...children];
}

function snapshot(home: string): HomeSnapshot {
  const digest = archiveDigest(home, snapshotPaths, [
    `--exclude=${snapshotExclusion}`,
  ]);
  const paths = snapshotPaths.flatMap((path) => observedPaths(home, path));
  const entries = Object.fromEntries(
    paths.map((path) => [
      path,
      archiveDigest(home, [path], ["--no-recursion"]),
    ]),
  );
  return { digest, entries };
}

function snapshotChange(
  before: string | undefined,
  after: string | undefined,
): string | undefined {
  if (before === after) {
    return undefined;
  }
  if (before === undefined) {
    return "added";
  }
  if (after === undefined) {
    return "removed";
  }
  return "modified";
}

function reportSnapshotChanges(
  before: HomeSnapshot,
  after: HomeSnapshot,
): void {
  const paths = [
    ...new Set([...Object.keys(before.entries), ...Object.keys(after.entries)]),
  ].toSorted();
  const changes = paths.flatMap((path) => {
    const kind = snapshotChange(before.entries[path], after.entries[path]);
    return kind === undefined ? [] : [`${kind}: ${JSON.stringify(path)}\n`];
  });
  process.stderr.write(
    changes.length === 0
      ? "Snapshot differs without a stable per-path delta\n"
      : changes.join(""),
  );
}

type ProfileOutput = Readonly<{ stdout: string; stderr: string }>;

function emitResult(result: ProfileOutput): void {
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
}

function profileCommand(command: readonly string[]): ProfileOutput {
  const result = checkSmokeCommand(command);
  return { stdout: result.stdout.toString(), stderr: result.stderr.toString() };
}

function verifyReplay(home: string, root: string): void {
  const before = snapshot(home);
  const repeat = profileCommand(installCommand);
  if (repeat.stdout.length > 0 || repeat.stderr.length > 0) {
    emitResult(repeat);
    throw new Error("Repeated installation must be silent");
  }
  const after = snapshot(home);
  if (after.digest !== before.digest) {
    reportSnapshotChanges(before, after);
    throw new Error("Installed artifacts changed on repeat installation");
  }
  verifyInstallation(home, root);
}

function homebrewLockRefreshCommand(root: string): string[] {
  return [
    "bun",
    "--config=/dev/null",
    "--no-env-file",
    "--no-install",
    join(root, "tooling/homebrew-lock.ts"),
    "refresh",
    join(root, "homebrew.lock.json"),
  ];
}

function main(): void {
  const home = pathSchema.parse(process.env.HOME);
  const root = resolve(import.meta.dir, "..");
  emitResult(profileCommand(homebrewLockRefreshCommand(root)));
  emitResult(profileCommand(installCommand));
  verifyInstallation(home, root);
  verifyReplay(home, root);
}

if (import.meta.main) {
  try {
    main();
  } catch (error) {
    reportCheckFailure(error);
  }
}
