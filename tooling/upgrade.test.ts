import { afterEach, expect, test } from "bun:test";
import {
  cleanupUpgradeFixtures,
  upgradeFixture,
} from "./upgrade-test-support.ts";
import { join } from "node:path";
import { rm } from "node:fs/promises";

afterEach(cleanupUpgradeFixtures);

test.each([
  "pull",
  "moon exec",
  "brew update",
  "brew upgrade --yes",
  "brew upgrade --cask",
  "brew cleanup",
  "brew doctor",
  "brew missing",
  "mas outdated",
  "mas upgrade",
  "npm update",
  "claude update",
  "claude plugin list",
  "claude plugin update first@fixture",
  "nvim --headless",
])("aggregates %s failure through the shipped entry point", async (failure) => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({ UPGRADE_FAILURE: failure });
  expect(result.output).toContain("simulated failure");
  expect(result.status).toBe(1);
  expect(result.calls).toContain("codex plugin marketplace upgrade");
});

test("reports success, unsupported sources and the original domain order", async () => {
  const fixture = await upgradeFixture();
  const result = await fixture.run();
  expect(result.status).toBe(0);
  const order = [
    "pull",
    "moon exec --quiet repository:install",
    "brew update",
    "brew upgrade --yes",
    "brew upgrade --cask --greedy --yes",
    "brew cleanup -s",
    "brew doctor",
    "brew missing",
    "mas outdated",
    "mas upgrade",
    "moon upgrade",
    "volta pin node@lts",
    "volta install node@24.19.0",
    "npm update -g",
    "claude update",
    "claude plugin list --json",
    "claude plugin update first@fixture",
    "claude plugin update second@fixture",
    "codex plugin marketplace upgrade",
    "nvim --headless +Lazy! sync",
  ];
  const offsets = order.map((invocation) => result.calls.indexOf(invocation));
  expect(offsets.every((offset) => offset >= 0)).toBe(true);
  expect(offsets).toEqual(offsets.toSorted((left, right) => left - right));
  expect(result.output).toContain("[succeeded] npm global packages upgrade");
  expect(result.output).toContain(
    "[unsupported] Codex local and host-managed sources",
  );
  expect(result.calls).not.toContain("marketplace update");
});

test.each(["brew", "mas", "npm", "claude", "codex", "nvim"])(
  "reports absent optional %s without success",
  async (command) => {
    const fixture = await upgradeFixture();
    await rm(join(fixture.bin, command));
    const result = await fixture.run();
    expect(result.status).toBe(0);
    expect(result.output).toContain(`${command} not found`);
    expect(result.output).toContain("[not-attempted]");
    expect(result.calls).not.toContain(`\n${command} `);
    expect(result.calls).toContain("volta install node@24.19.0");
    if (command !== "nvim") {
      expect(result.calls).toContain("nvim --headless");
    }
  },
);

test.each([
  "moon upgrade",
  "volta pin",
  "volta install",
  "codex plugin marketplace upgrade",
])("retains aggregation for %s", async (failure) => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({ UPGRADE_FAILURE: failure });
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed]");
  expect(result.calls).toContain("nvim --headless");
});

test("continues other Claude plugins after a partial failure", async () => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({
    UPGRADE_FAILURE: "claude plugin update first@fixture",
  });
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Claude plugin first@fixture");
  expect(result.output).toContain("[succeeded] Claude plugin second@fixture");
});

test.each([
  "null",
  "{}",
  "not json",
  '[{"id":"ok"},{}]',
  '[{"id":"--help"}]',
  '[{"id":""}]',
])("refuses invalid plugin inventory %s", async (plugins) => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({ UPGRADE_PLUGINS: plugins });
  expect(result.status).toBe(1);
  expect(result.calls).not.toContain("claude plugin update");
  expect(result.output).toContain("[failed] Claude plugin inventory");
  expect(result.output).toContain("[not-attempted] Claude plugin updates");
  expect(result.calls).toContain("codex plugin marketplace upgrade");
});

test("missing Moon fails required operations while later domains continue", async () => {
  const fixture = await upgradeFixture();
  await rm(join(fixture.home, ".moon/bin/moon"));
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("[failed] Dotfiles redeployment");
  expect(result.output).toContain("[failed] Moon upgrade");
  expect(result.calls).toContain("brew update");
  expect(result.calls).toContain("nvim --headless");
});

test("missing Bun stops at bootstrap without attempting any upgrade", async () => {
  const fixture = await upgradeFixture();
  await rm(join(fixture.bin, "bun"));
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("bun not found, unable to run upgrade");
  expect(result.calls).toBe("");
});

test("empty plugin inventory is not reported as updated", async () => {
  const fixture = await upgradeFixture();
  const result = await fixture.run({ UPGRADE_PLUGINS: "[]" });
  expect(result.status).toBe(0);
  expect(result.output).toContain(
    "[not-attempted] Claude plugin updates: no installed plugins",
  );
  expect(result.calls).not.toContain("claude plugin update");
});

test("computes three-day quarantine in UTC without the BSD date utility", async () => {
  const fixture = await upgradeFixture();
  const quarantineDays = 3;
  const millisecondsPerDay = 86_400_000;
  const oldest = Date.now() - quarantineDays * millisecondsPerDay;
  const result = await fixture.run();
  const newest = Date.now() - quarantineDays * millisecondsPerDay;
  const match =
    /npm update -g --before=(?<cutoff>\S+) --min-release-age-exclude=@openai\/codex/u.exec(
      result.calls,
    );
  expect(match).not.toBeNull();
  const cutoff = Date.parse(match?.groups?.cutoff ?? "");
  const millisecondsPerSecond = 1000;
  expect(cutoff).toBeGreaterThanOrEqual(oldest - millisecondsPerSecond);
  expect(cutoff).toBeLessThanOrEqual(newest);
  expect(
    result.calls.split("\n").some((call) => call.startsWith("date ")),
  ).toBe(false);
});
