import { commitNeovimLockfile, upgradeRepository } from "./upgrade-git.ts";
import type { UpgradeRunner } from "./upgrade-runner.ts";
import { createUpgradeRunner } from "./upgrade-runner.ts";
import { join } from "node:path";
import { upgradeNode } from "./upgrade-node.ts";
import { z } from "zod";

const environmentSchema = z.object({ HOME: z.string().min(1) });
const pluginsSchema = z.array(
  z.object({
    id: z
      .string()
      .min(1)
      .regex(/^[^-\s]\S*$/u),
  }),
);

function homebrewLockCommand(
  repository: string,
  operation: "refresh" | "verify-installed",
): string[] {
  return [
    process.execPath,
    "--config=/dev/null",
    "--no-env-file",
    "--no-install",
    join(import.meta.dir, "homebrew-lock.ts"),
    operation,
    join(repository, "homebrew.lock.json"),
  ];
}

async function upgradePackages(
  runner: UpgradeRunner,
  repository: string,
): Promise<void> {
  process.stdout.write("ℹ️  Brew\n");
  const metadata = await runner.command("Brew metadata", ["brew", "update"]);
  if (metadata.state === "succeeded") {
    await runner.command(
      "Homebrew lock refresh",
      homebrewLockCommand(repository, "refresh"),
    );
  } else {
    runner.skip("Homebrew lock refresh", "Homebrew metadata unavailable");
  }
  await runner.command("Brew packages", [
    "brew",
    "upgrade",
    "--formula",
    "--yes",
  ]);
  await runner.command("Brew casks", [
    "brew",
    "upgrade",
    "--cask",
    "--greedy",
    "--yes",
  ]);
  if (metadata.state === "not-attempted") {
    runner.skip("Homebrew lock verification", "Homebrew unavailable");
  } else {
    await runner.command(
      "Homebrew lock verification",
      homebrewLockCommand(repository, "verify-installed"),
    );
  }
  await runner.command("Brew cleanup", ["brew", "cleanup", "-s"], {
    env: { HOMEBREW_CLEANUP_MAX_AGE_DAYS: "7" },
  });
  process.stdout.write("ℹ️  Brew diagnostic\n");
  await runner.command("Brew doctor", ["brew", "doctor"]);
  await runner.command("Brew missing", ["brew", "missing"]);
  process.stdout.write("ℹ️  Mas upgrade\n");
  await runner.command("Mas outdated", ["mas", "outdated"]);
  await runner.command("Mas upgrade", ["mas", "upgrade"]);
}

async function upgradeClaudePlugins(runner: UpgradeRunner): Promise<void> {
  process.stdout.write("ℹ️  Claude plugins update\n");
  const listing = await runner.command(
    "Claude plugin list",
    ["claude", "plugin", "list", "--json"],
    { capture: true },
  );
  if (listing.state !== "succeeded") {
    runner.skip("Claude plugin updates", "plugin inventory unavailable");
    return;
  }
  const inventory = await runner.attempt("Claude plugin inventory", () =>
    pluginsSchema.parse(JSON.parse(listing.value)),
  );
  if (inventory.state !== "succeeded") {
    runner.skip("Claude plugin updates", "invalid plugin inventory");
    return;
  }
  if (inventory.value.length === 0) {
    runner.skip("Claude plugin updates", "no installed plugins");
    return;
  }
  for (const plugin of inventory.value) {
    await runner.command(
      `Claude plugin ${plugin.id}`,
      ["claude", "plugin", "update", plugin.id],
      { required: true },
    );
  }
}

async function upgradeAgents(runner: UpgradeRunner): Promise<void> {
  process.stdout.write("ℹ️  Claude Code CLI update\n");
  await runner.command("Claude Code CLI update", ["claude", "update"]);
  await upgradeClaudePlugins(runner);
  process.stdout.write("ℹ️  Codex plugins update\n");
  runner.unsupported(
    "Codex local and host-managed sources",
    "not refreshable; skipped",
  );
  await runner.command("Codex Git marketplace refresh command", [
    "codex",
    "plugin",
    "marketplace",
    "upgrade",
  ]);
}

async function main(): Promise<number> {
  const { HOME: home } = environmentSchema.parse(process.env);
  const repository = join(home, ".dotfiles");
  const moon = join(home, ".moon/bin/moon");
  const runner = createUpgradeRunner();
  process.stdout.write("ℹ️  Dotfiles\n");
  await upgradeRepository(runner, repository, moon);
  await upgradePackages(runner, repository);
  process.stdout.write("ℹ️  Volta Node.js upgrade\n");
  await upgradeNode(runner, repository);
  process.stdout.write("ℹ️  Codex CLI update\n");
  await runner.command("Codex CLI update", [
    "npm",
    "update",
    "-g",
    "@openai/codex",
  ]);
  await upgradeAgents(runner);
  process.stdout.write("ℹ️  NeoVim plugins\n");
  const synced = await runner.command(
    "NeoVim plugins",
    [
      "nvim",
      "--headless",
      "+Lazy! sync",
      "+lua local ok, err = pcall(dofile, vim.env.DOTFILES_UPGRADE_NVIM); if not ok then vim.api.nvim_err_writeln(tostring(err)); vim.cmd('cquit 1') end",
      "+qa",
    ],
    {
      env: {
        DOTFILES_UPGRADE_NVIM: join(
          import.meta.dir,
          "../home/.config/nvim/upgrade.lua",
        ),
      },
    },
  );
  if (synced.state === "succeeded") {
    await commitNeovimLockfile(runner, repository);
  } else {
    runner.skip("NeoVim lockfile commit", "plugin sync did not succeed");
  }
  return runner.finish();
}

try {
  process.exitCode = await main();
} catch (error) {
  process.stderr.write(
    `upgrade: ${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
