import { commandResult, upgradeFixture } from "./upgrade-test-support.ts";
import { cp, mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const fixtureConfiguration = `
vim.opt.rtp:prepend(vim.env.UPGRADE_LAZY)
require("lazy").setup({
  spec = {
    { name = "lazy.nvim", dir = vim.env.UPGRADE_LAZY, pin = true },
    { name = "first", url = "file://" .. vim.env.UPGRADE_PLUGIN_ROOT .. "/first", submodules = false },
    { name = "second", url = "file://" .. vim.env.UPGRADE_PLUGIN_ROOT .. "/second", submodules = false, build = function()
      if vim.env.UPGRADE_BUILD_FAILURE == "1" then error("fixture build failed") end
    end },
  },
  root = vim.env.HOME .. "/plugins",
  defaults = { lazy = true },
  lockfile = vim.env.HOME .. "/.dotfiles/home/.config/nvim/lazy-lock.json",
  install = { missing = false },
  checker = { enabled = false },
  change_detection = { enabled = false },
  rocks = { enabled = false },
  pkg = { enabled = false },
  git = { filter = false },
})
`;

let downloadedSource: string | null = null;
let lazySource: string | null = null;
const gitEnvironment = {
  ...process.env,
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_TERMINAL_PROMPT: "0",
};

async function prepareNeovimIntegration(): Promise<void> {
  if (Bun.which("nvim") === null) {
    throw new Error("Neovim integration requires nvim on PATH");
  }
  const git = Bun.which("git");
  if (git === null) {
    throw new Error("Neovim integration requires git on PATH");
  }
  const lock = z
    .object({
      "lazy.nvim": z.object({ commit: z.string().regex(/^[a-f0-9]{40}$/u) }),
    })
    .parse(
      await Bun.file(
        join(import.meta.dir, "../home/.config/nvim/lazy-lock.json"),
      ).json(),
    );
  lazySource = process.env.UPGRADE_TEST_LAZY_DIR ?? null;
  if (lazySource === null) {
    downloadedSource = await mkdtemp(join(tmpdir(), "upgrade-lazy-source-"));
    lazySource = downloadedSource;
    await checkoutLockedLazy(git, lazySource, lock["lazy.nvim"].commit);
  }
  const revision = await commandResult(
    [git, "-C", lazySource, "rev-parse", "HEAD"],
    lazySource,
    gitEnvironment,
  );
  if (
    revision.status !== 0 ||
    revision.stdout.trim() !== lock["lazy.nvim"].commit
  ) {
    throw new Error(
      "UPGRADE_TEST_LAZY_DIR must match the repository lazy.nvim lock",
    );
  }
}

async function cleanupNeovimIntegration(): Promise<void> {
  if (downloadedSource !== null) {
    await rm(downloadedSource, { recursive: true, force: true });
  }
}

async function neovimUpgradeFixture(): Promise<NeovimFixture> {
  const source = lazySource;
  const nvim = Bun.which("nvim");
  if (source === null || nvim === null) {
    throw new Error(
      "Neovim integration requires nvim and UPGRADE_TEST_LAZY_DIR at the locked lazy.nvim revision",
    );
  }
  const fixture = await upgradeFixture();
  const lazy = join(fixture.root, "lazy.nvim");
  await cp(source, lazy, { recursive: true });
  await rm(join(fixture.bin, "nvim"));
  await symlink(nvim, join(fixture.bin, "nvim"));
  const config = join(fixture.home, ".config/nvim");
  await mkdir(config, { recursive: true });
  await cp(
    join(import.meta.dir, "../home/.config/nvim/upgrade.lua"),
    join(fixture.repository, "home/.config/nvim/upgrade.lua"),
  );
  await initializePlugins(fixture);
  await Bun.write(
    join(fixture.repository, "home/.config/nvim/lazy-lock.json"),
    "{}\n",
  );
  await Bun.write(join(config, "init.lua"), fixtureConfiguration);
  const env = {
    ...fixture.env,
    UPGRADE_LAZY: lazy,
    UPGRADE_PLUGIN_ROOT: fixture.root,
  };
  return {
    ...fixture,
    config,
    run: (
      overrides: Readonly<Record<string, string>> = {},
    ): ReturnType<BaseFixture["run"]> => fixture.run({ ...env, ...overrides }),
  };
}

type BaseFixture = Awaited<ReturnType<typeof upgradeFixture>>;
interface NeovimFixture extends BaseFixture {
  readonly config: string;
}

async function initializePlugins(fixture: BaseFixture): Promise<void> {
  for (const name of ["first", "second"]) {
    const plugin = join(fixture.root, name);
    await mkdir(plugin);
    await Bun.write(join(plugin, "plugin.txt"), name);
    for (const args of [
      ["init", "-b", "main"],
      ["add", "."],
      [
        "-c",
        "user.email=fixture@example.test",
        "-c",
        "user.name=Fixture",
        "commit",
        "-qm",
        "base",
      ],
    ]) {
      const result = await fixture.git("-C", plugin, ...args);
      if (result.status !== 0) {
        throw new Error(result.output);
      }
    }
  }
}

async function checkoutLockedLazy(
  git: string,
  source: string,
  commit: string,
): Promise<void> {
  for (const args of [
    ["init"],
    ["fetch", "--depth=1", "https://github.com/folke/lazy.nvim.git", commit],
    ["checkout", "--detach", "FETCH_HEAD"],
  ]) {
    const result = await commandResult(
      [git, "-c", "core.hooksPath=/dev/null", "-C", source, ...args],
      source,
      gitEnvironment,
    );
    if (result.status !== 0) {
      throw new Error(result.output);
    }
  }
}

export {
  cleanupNeovimIntegration,
  neovimUpgradeFixture,
  prepareNeovimIntegration,
};
