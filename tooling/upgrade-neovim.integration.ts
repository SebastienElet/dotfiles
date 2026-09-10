import {
  afterAll,
  afterEach,
  beforeAll,
  expect,
  setDefaultTimeout,
  test,
} from "bun:test";
import { appendFile, rm } from "node:fs/promises";
import {
  cleanupNeovimIntegration,
  neovimUpgradeFixture,
  prepareNeovimIntegration,
} from "./upgrade-neovim-test-support.ts";
import { cleanupUpgradeFixtures } from "./upgrade-test-support.ts";
import { join } from "node:path";

const preparationTimeout = 60_000;
const integrationTimeout = 30_000;
setDefaultTimeout(integrationTimeout);
beforeAll(prepareNeovimIntegration, preparationTimeout);
afterAll(cleanupNeovimIntegration);
afterEach(cleanupUpgradeFixtures);

test("shipped upgrade refuses the lockfile after a real Lazy build failure", async () => {
  const fixture = await neovimUpgradeFixture();
  const result = await fixture.run({ UPGRADE_BUILD_FAILURE: "1" });
  expect(result.output).toContain("fixture build failed");
  expect(result.status).toBe(1);
  expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
  expect(result.calls).not.toContain(" commit");
});

test("real Lazy success commits the lockfile and leaves the remote unchanged", async () => {
  const fixture = await neovimUpgradeFixture();
  const before = await fixture.git("ls-remote", "origin");
  const result = await fixture.run();
  expect(result.status).toBe(0);
  expect(result.output).toContain("[succeeded] NeoVim lockfile commit");
  const committed = await fixture.git(
    "show",
    "--format=",
    "--name-only",
    "HEAD",
  );
  const after = await fixture.git("ls-remote", "origin");
  expect(committed.stdout.trim()).toBe("home/.config/nvim/lazy-lock.json");
  expect(after.stdout).toBe(before.stdout);
});

test.each(["missing", "invalid Lua"])(
  "refuses %s verifier before committing",
  async (failure) => {
    const fixture = await neovimUpgradeFixture();
    const helper = join(fixture.repository, "home/.config/nvim/upgrade.lua");
    await (failure === "missing"
      ? rm(helper)
      : Bun.write(helper, "this is invalid Lua"));
    const result = await fixture.run();
    expect(result.status).toBe(1);
    expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
    expect(result.calls).not.toContain(" commit");
  },
);

test.each([
  [
    "task evidence missing",
    "for _, plugin in pairs(config.plugins) do plugin._.tasks = nil end",
  ],
  ["task API missing", "config.plugins.first._.tasks[1].has_errors = false"],
  [
    "task probe empty",
    "config.plugins.first._.tasks[1].has_errors = function() return nil end",
  ],
  [
    "task probe throws",
    'config.plugins.first._.tasks[1].has_errors = function() error("fixture probe failed") end',
  ],
  [
    "task still running",
    "config.plugins.first._.tasks[1].running = function() return true end",
  ],
])("refuses %s after real Lazy sync", async (_failure, disruption) => {
  const fixture = await neovimUpgradeFixture();
  await appendFile(
    join(fixture.config, "init.lua"),
    `
vim.api.nvim_create_autocmd("User", { pattern = "LazySync", callback = function()
  local config = require("lazy.core.config")
  ${disruption}
end })
`,
  );
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
  expect(result.calls).not.toContain(" commit");
});

test("refuses unavailable Lazy initialization", async () => {
  const fixture = await neovimUpgradeFixture();
  await Bun.write(join(fixture.config, "init.lua"), "");
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
  expect(result.calls).not.toContain(" commit");
});
