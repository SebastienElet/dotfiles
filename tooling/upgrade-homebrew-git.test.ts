import { afterEach, expect, test } from "bun:test";
import {
  cleanupUpgradeFixtures,
  upgradeFixture,
} from "./upgrade-test-support.ts";
import { join } from "node:path";
import { rm } from "node:fs/promises";

afterEach(cleanupUpgradeFixtures);

const jsonIndentation = 2;

async function upgradeFixtureWithHomebrewLock(): Promise<
  Awaited<ReturnType<typeof upgradeFixture>>
> {
  const fixture = await upgradeFixture();
  await Bun.write(
    join(fixture.repository, "homebrew.lock.json"),
    `${JSON.stringify({ formulae: { awscli: "1.0.0" }, casks: {} }, null, jsonIndentation)}\n`,
  );
  return fixture;
}

test("commits the refreshed Homebrew lock before Neovim without including staged changes", async () => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  await Bun.write(join(fixture.repository, "unrelated"), "staged\n");
  await fixture.git("add", "unrelated");
  const remoteBefore = await fixture.git("ls-remote", "origin");
  const result = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
  expect(result.status).toBe(0);
  const log = await fixture.git("log", "-2", "--format=%s");
  expect(log.stdout.trim().split("\n")).toEqual([
    "chore(nvim): update plugins",
    "chore(homebrew): update lockfile",
  ]);
  const committed = await fixture.git(
    "show",
    "--format=",
    "--name-only",
    "HEAD~1",
  );
  expect(committed.stdout.trim()).toBe("homebrew.lock.json");
  const lock = await fixture.git("show", "HEAD~1:homebrew.lock.json");
  expect(JSON.parse(lock.stdout)).toEqual({
    formulae: { awscli: "2.0.0" },
    casks: {},
  });
  const staged = await fixture.git("diff", "--cached", "--name-only");
  expect(staged.stdout.trim()).toBe("unrelated");
  const remoteAfter = await fixture.git("ls-remote", "origin");
  expect(remoteAfter.stdout).toBe(remoteBefore.stdout);
});

test("replaying unchanged Homebrew and Neovim locks creates no commit", async () => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  const result = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
  expect(result.status).toBe(0);
  const beforeReplay = await fixture.git("rev-parse", "HEAD");
  const replay = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
  expect(replay.status).toBe(0);
  const afterReplay = await fixture.git("rev-parse", "HEAD");
  expect(afterReplay.stdout).toBe(beforeReplay.stdout);
  expect(replay.output).toContain(
    "[not-attempted] Homebrew lockfile commit: lockfile unchanged",
  );
  expect(replay.calls).not.toContain(" commit");
});

test.each([
  "brew update",
  "brew info --json=v2 --formula awscli",
  "brew info --json=v2 --installed",
])("does not commit the Homebrew lock after %s fails", async (failure) => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  const result = await fixture.run({
    UPGRADE_BREW_VERSION: "2.0.0",
    UPGRADE_FAILURE: failure,
  });
  expect(result.status).toBe(1);
  expect(result.output).toContain("[not-attempted] Homebrew lockfile commit");
  const log = await fixture.git("log", "--format=%s");
  expect(log.stdout.trim().split("\n")).toEqual([
    "chore(nvim): update plugins",
    "base",
  ]);
});

test.each([
  "status --porcelain -- homebrew.lock.json",
  "commit -m chore(homebrew): update lockfile",
])("keeps Homebrew %s failures visible and replayable", async (failure) => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  const result = await fixture.run({
    UPGRADE_BREW_VERSION: "2.0.0",
    UPGRADE_FAILURE: failure,
  });
  expect(result.status).toBe(1);
  expect(result.output).toContain("simulated failure");
  const log = await fixture.git("log", "--format=%s");
  expect(log.stdout.trim().split("\n")).toEqual([
    "chore(nvim): update plugins",
    "base",
  ]);
  const replay = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
  expect(replay.status).toBe(0);
  const recovered = await fixture.git("show", "--format=", "--name-only");
  expect(recovered.stdout.trim()).toBe("homebrew.lock.json");
});

test.each(["feature", "detached"])(
  "keeps the refreshed Homebrew lock uncommitted on %s",
  async (branch) => {
    const fixture = await upgradeFixtureWithHomebrewLock();
    await fixture.git(
      "checkout",
      ...(branch === "detached" ? ["--detach"] : ["-b", branch]),
    );
    const before = await fixture.git("rev-parse", "HEAD");
    const result = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
    expect(result.status).toBe(0);
    const after = await fixture.git("rev-parse", "HEAD");
    expect(after.stdout).toBe(before.stdout);
    expect(result.output).toContain("[not-attempted] Homebrew lockfile commit");
    const lock: unknown = await Bun.file(
      join(fixture.repository, "homebrew.lock.json"),
    ).json();
    expect(lock).toEqual({ formulae: { awscli: "2.0.0" }, casks: {} });
  },
);

test("keeps a verified Homebrew commit when Neovim sync fails", async () => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  const result = await fixture.run({
    UPGRADE_BREW_VERSION: "2.0.0",
    UPGRADE_FAILURE: "nvim --headless",
  });
  expect(result.status).toBe(1);
  const log = await fixture.git("log", "--format=%s");
  expect(log.stdout.trim().split("\n")).toEqual([
    "chore(homebrew): update lockfile",
    "base",
  ]);
});

test("skips the Homebrew commit when the dotfiles Git repository is absent", async () => {
  const fixture = await upgradeFixtureWithHomebrewLock();
  await rm(join(fixture.repository, ".git"), { recursive: true });
  await rm(join(fixture.bin, "nvim"));
  const result = await fixture.run({ UPGRADE_BREW_VERSION: "2.0.0" });
  expect(result.status).toBe(0);
  expect(result.output).toContain(
    "[not-attempted] Homebrew lockfile commit: repository unavailable",
  );
  expect(result.output).toContain(
    "[not-attempted] NeoVim lockfile commit: plugin sync did not succeed",
  );
  expect(result.calls).not.toContain(" commit");
});
