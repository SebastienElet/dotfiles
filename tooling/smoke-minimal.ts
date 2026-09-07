import { accessSync, constants } from "node:fs";
import { checkCommand, reportCheckFailure } from "./check-command.ts";
import { join, resolve } from "node:path";
import { createHash } from "node:crypto";
import { z } from "zod";

const pathSchema = z.string().min(1);
const snapshotPaths = [
  ".agents/skills",
  ".arnes.yaml",
  ".claude",
  ".claude.json",
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
  ".local/bin/agent-memory",
  ".local/bin/arnes",
  ".local/bin/claude",
  ".local/bin/colgrep-search",
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

function verifyInstallation(home: string, root: string): void {
  checkCommand([
    "brew",
    "bundle",
    "check",
    "--quiet",
    "--no-upgrade",
    "--file",
    join(root, "Brewfile"),
  ]);
  const prefix = pathSchema.parse(
    checkCommand(["brew", "--prefix"]).stdout.toString().trim(),
  );
  const executables = [
    join(prefix, "bin/colgrep"),
    ...[
      "agent-handoff",
      "agent-memory",
      "arnes",
      "claude",
      "colgrep-search",
    ].map((command) => join(home, ".local/bin", command)),
    ...["codex", "node", "pnpm"].map((command) =>
      join(home, ".volta/bin", command),
    ),
  ];
  for (const executable of executables) {
    accessSync(executable, constants.X_OK);
  }
  checkCommand(
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

function snapshot(home: string): string {
  const result = checkCommand(["tar", "-cf", "-", ...snapshotPaths], {
    cwd: home,
  });
  return createHash("sha256").update(result.stdout).digest("hex");
}

function main(): void {
  const home = pathSchema.parse(process.env.HOME);
  const root = resolve(import.meta.dir, "..");
  const first = checkCommand(installCommand);
  process.stdout.write(first.stdout);
  process.stderr.write(first.stderr);
  verifyInstallation(home, root);
  const before = snapshot(home);
  const repeat = checkCommand(installCommand);
  if (repeat.stdout.length > 0 || repeat.stderr.length > 0) {
    process.stdout.write(repeat.stdout);
    process.stderr.write(repeat.stderr);
    throw new Error("Repeated installation must be silent");
  }
  if (snapshot(home) !== before) {
    throw new Error("Installed artifacts changed on repeat installation");
  }
  verifyInstallation(home, root);
}

try {
  main();
} catch (error) {
  reportCheckFailure(error);
}
