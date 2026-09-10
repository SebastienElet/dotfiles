import { afterEach, expect, test } from "bun:test";
import {
  cleanupUpgradeFixtures,
  upgradeFixture,
} from "./upgrade-test-support.ts";
import { join } from "node:path";
import { rm } from "node:fs/promises";

afterEach(cleanupUpgradeFixtures);

test("missing central branch executable prevents Git mutations and continues upgrades", async () => {
  const fixture = await upgradeFixture();
  await rm(join(fixture.repository, "tooling/git-main-branch"));
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Dotfiles main branch");
  expect(result.output).toContain("[not-attempted] Dotfiles repository update");
  expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
  expect(result.calls).not.toContain(" pull");
  expect(result.calls).not.toContain(" commit");
  expect(result.calls).toContain("brew update");
  expect(result.calls).toContain("nvim --headless");
});

test.each(["feature", "detached"])(
  "protects %s while leaving plugin changes available",
  async (branch) => {
    const fixture = await upgradeFixture();
    await fixture.git(
      "checkout",
      ...(branch === "detached" ? ["--detach"] : ["-b", branch]),
    );
    const before = await fixture.git("rev-parse", "HEAD");
    const result = await fixture.run();
    expect(result.status).toBe(0);
    const after = await fixture.git("rev-parse", "HEAD");
    expect(after.stdout).toBe(before.stdout);
    expect(result.calls).not.toContain(" pull");
    expect(result.calls).not.toContain(" commit");
    expect(result.calls).not.toContain(" push");
    expect(result.calls).not.toContain("moon exec");
    expect(result.output).toContain(
      "[not-attempted] Dotfiles repository update",
    );
    expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
    expect(
      await Bun.file(
        join(fixture.repository, "home/.config/nvim/lazy-lock.json"),
      ).text(),
    ).toBe("updated\n");
  },
);

test.each(["branch --show-current", "show-ref", "rev-parse"])(
  "fails closed on branch probe %s",
  async (failure) => {
    const fixture = await upgradeFixture();
    const result = await fixture.run({ UPGRADE_FAILURE: failure });
    expect(result.status).toBe(1);
    expect(result.output).toContain("simulated failure");
    expect(result.calls).not.toContain(" pull");
    expect(result.calls).not.toContain(" commit");
    expect(result.calls).toContain("brew update");
  },
);

test.each(["main", "master", "trunk"])(
  "uses central resolution for %s and commits only the lockfile",
  async (branch) => {
    const fixture = await upgradeFixture();
    await fixture.git("branch", "-m", branch);
    await fixture.git("push", "-u", "origin", branch);
    await Bun.write(join(fixture.repository, "unrelated"), "staged\n");
    await fixture.git("add", "unrelated");
    const remoteBefore = await fixture.git("ls-remote", "origin");
    const result = await fixture.run();
    expect(result.status).toBe(0);
    const log = await fixture.git("log", "-1", "--format=%s");
    expect(log.stdout.trim()).toBe("chore(nvim): update plugins");
    const committed = await fixture.git(
      "show",
      "--format=",
      "--name-only",
      "HEAD",
    );
    expect(committed.stdout.trim()).toBe("home/.config/nvim/lazy-lock.json");
    const staged = await fixture.git("diff", "--cached", "--name-only");
    expect(staged.stdout.trim()).toBe("unrelated");
    const remoteAfter = await fixture.git("ls-remote", "origin");
    expect(remoteAfter.stdout).toBe(remoteBefore.stdout);
  },
);

test("failed pull leaves deployment unattempted and other domains run", async () => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({ UPGRADE_FAILURE: "pull" });
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Dotfiles repository update");
  expect(result.output).toContain("[not-attempted] Dotfiles redeployment");
  expect(result.calls).not.toContain("moon exec");
  expect(result.calls).toContain("brew update");
});

test("failed sync does not commit a partial lockfile", async () => {
  const fixture = await upgradeFixture();
  await Bun.write(
    join(fixture.repository, "home/.config/nvim/lazy-lock.json"),
    "partial\n",
  );
  const result = await fixture.run({ UPGRADE_FAILURE: "nvim --headless" });
  expect(result.status).toBe(1);
  expect(result.calls).not.toContain(" commit");
  expect(result.output).toContain("[not-attempted] NeoVim lockfile commit");
});

test.each(["status --porcelain", "commit -m"])(
  "aggregates lockfile %s failures",
  async (failure) => {
    const fixture = await upgradeFixture();
    const result = await fixture.run({ UPGRADE_FAILURE: failure });
    expect(result.status).toBe(1);
    expect(result.output).toContain("simulated failure");
  },
);

test("replay after failed commit saves once and never pushes", async () => {
  const fixture = await upgradeFixture();
  const failed = await fixture.run({ UPGRADE_FAILURE: "commit -m" });
  expect(failed.status).toBe(1);
  const recovered = await fixture.run();
  expect(recovered.status).toBe(0);
  const before = await fixture.git("rev-parse", "HEAD");
  const replay = await fixture.run();
  expect(replay.status).toBe(0);
  const after = await fixture.git("rev-parse", "HEAD");
  expect(after.stdout).toBe(before.stdout);
  expect(replay.output).toContain(
    "[not-attempted] NeoVim lockfile commit: lockfile unchanged",
  );
  expect(replay.calls).not.toContain(" commit");
  expect(replay.calls).not.toContain(" push");
});
