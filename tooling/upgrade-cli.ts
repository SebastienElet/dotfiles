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
const quarantineDays = 3;
const millisecondsPerDay = 86_400_000;

async function upgradePackages(
  runner: UpgradeRunner,
  moon: string,
): Promise<void> {
  process.stdout.write("ℹ️  Brew\n");
  await runner.command("Brew metadata", ["brew", "update"]);
  await runner.command("Brew packages", ["brew", "upgrade", "--yes"]);
  await runner.command("Brew casks", [
    "brew",
    "upgrade",
    "--cask",
    "--greedy",
    "--yes",
  ]);
  await runner.command("Brew cleanup", ["brew", "cleanup", "-s"], {
    env: { HOMEBREW_CLEANUP_MAX_AGE_DAYS: "7" },
  });
  process.stdout.write("ℹ️  Brew diagnostic\n");
  await runner.command("Brew doctor", ["brew", "doctor"]);
  await runner.command("Brew missing", ["brew", "missing"]);
  process.stdout.write("ℹ️  Mas upgrade\n");
  await runner.command("Mas outdated", ["mas", "outdated"]);
  await runner.command("Mas upgrade", ["mas", "upgrade"]);
  process.stdout.write("ℹ️  Moon upgrade\n");
  await runner.command("Moon upgrade", [moon, "upgrade"], { required: true });
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
  await upgradePackages(runner, moon);
  process.stdout.write("ℹ️  Volta Node.js upgrade\n");
  await upgradeNode(runner, repository);
  process.stdout.write("ℹ️  npm global packages upgrade\n");
  const cutoff = new Date(
    Date.now() - quarantineDays * millisecondsPerDay,
  ).toISOString();
  await runner.command("npm global packages upgrade", [
    "npm",
    "update",
    "-g",
    `--before=${cutoff}`,
    "--min-release-age-exclude=@openai/codex",
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
