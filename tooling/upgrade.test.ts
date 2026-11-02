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
  "brew upgrade --formula --yes",
  "brew upgrade --cask",
  "brew info --json=v2 --installed",
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
    "curl -fsSL",
    "moon exec --quiet repository:install",
    "brew update",
    "brew upgrade --formula --yes",
    "brew upgrade --cask --greedy --yes",
    "brew info --json=v2 --installed",
    "brew cleanup -s",
    "brew doctor",
    "brew missing",
    "mas outdated",
    "mas upgrade",
    "volta pin node@lts",
    "volta install node@24.19.0",
    "npm update -g @openai/codex",
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
  expect(result.output).toContain("[succeeded] Codex CLI update");
  expect(result.output).toContain(
    "[unsupported] Codex local and host-managed sources",
  );
  expect(result.calls).not.toContain("marketplace update");
  expect(result.calls).not.toContain("moon upgrade");
});

test("redeploys with the Moon pin from a pulled checkout and keeps that version", async () => {
  const fixture = await upgradeFixture();
  await fixture.publishMoonPin("2.5.4");
  const result = await fixture.run();
  expect(result.status).toBe(0);
  expect(result.calls).toContain("moon 2.5.4 exec --quiet repository:install");
  expect(result.calls).not.toContain("moon upgrade");
  expect(await Bun.file(join(fixture.home, ".moon/bin/moon")).text()).toContain(
    "2.5.4",
  );
});

test("rejects an invalid pulled Moon pin before redeployment", async () => {
  const fixture = await upgradeFixture();
  await fixture.publishMoonPin("not-a-version");
  const result = await fixture.run();
  expect(result.status).toBe(1);
  expect(result.output).toContain("invalid Moon pin");
  expect(result.output).toContain("[failed] Moon bootstrap");
  expect(result.calls).not.toContain("moon exec");
});

test.each([
  "curl",
  "shasum",
  "unzip",
  "bun install",
  "install -m",
  "moon --version",
])(
  "refuses redeployment when Moon acquisition or validation fails at %s",
  async (failure) => {
    const fixture = await upgradeFixture();
    const result = await fixture.run({ UPGRADE_FAILURE: failure });
    expect(result.status).toBe(1);
    expect(result.output).toContain("[failed] Moon bootstrap");
    expect(result.output).toContain("[not-attempted] Dotfiles redeployment");
    expect(result.calls).not.toContain("moon exec");
  },
);

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

test.each(["volta pin", "volta install", "codex plugin marketplace upgrade"])(
  "retains aggregation for %s",
  async (failure) => {
    const fixture = await upgradeFixture();
    const result = await fixture.run({ UPGRADE_FAILURE: failure });
    expect(result.status).toBe(1);
    expect(result.output).toContain("[failed]");
    expect(result.calls).toContain("nvim --headless");
  },
);

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

test("missing Moon is restored before redeployment", async () => {
  const fixture = await upgradeFixture();
  await rm(join(fixture.home, ".moon/bin/moon"));
  const result = await fixture.run();
  expect(result.status).toBe(0);
  expect(result.output).toContain("[succeeded] Moon bootstrap");
  expect(result.calls).toContain("moon exec");
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

test("updates only the unpinned Codex CLI among npm global packages", async () => {
  const fixture = await upgradeFixture();
  const result = await fixture.run();
  expect(
    result.calls.split("\n").filter((call) => call.startsWith("npm ")),
  ).toEqual(["npm update -g @openai/codex"]);
});

const lockedBeforeUpgrade = '{"formulae":{"git":"1.0"},"casks":{}}\n';
const catalogWithNewerGit = JSON.stringify({
  formulae: [
    {
      name: "git",
      full_name: "git",
      versions: { stable: "2.0" },
      revision: 0,
      linked_keg: null,
      installed: [],
    },
  ],
  casks: [],
});

test.each(["brew upgrade --formula --yes", "brew upgrade --cask"])(
  "keeps the Homebrew lock unchanged when %s fails",
  async (failure) => {
    const fixture = await upgradeFixture();
    const lock = join(fixture.repository, "homebrew.lock.json");
    await Bun.write(lock, lockedBeforeUpgrade);
    const result = await fixture.run({
      UPGRADE_CATALOG: catalogWithNewerGit,
      UPGRADE_FAILURE: failure,
    });
    expect(result.status).toBe(1);
    expect(await Bun.file(lock).text()).toBe(lockedBeforeUpgrade);
    expect(result.output).toContain("[not-attempted] Homebrew lock refresh");
  },
);

test("keeps the Homebrew lock unchanged when the Brew metadata update fails", async () => {
  const fixture = await upgradeFixture();
  const lock = join(fixture.repository, "homebrew.lock.json");
  await Bun.write(lock, lockedBeforeUpgrade);
  const result = await fixture.run({
    UPGRADE_CATALOG: catalogWithNewerGit,
    UPGRADE_FAILURE: "brew update",
  });
  expect(result.status).toBe(1);
  expect(await Bun.file(lock).text()).toBe(lockedBeforeUpgrade);
  expect(result.output).toContain("[not-attempted] Homebrew lock refresh");
});

test("rewrites the Homebrew lock only after both upgrades succeed", async () => {
  const fixture = await upgradeFixture();
  const lock = join(fixture.repository, "homebrew.lock.json");
  await Bun.write(lock, lockedBeforeUpgrade);
  const result = await fixture.run({ UPGRADE_CATALOG: catalogWithNewerGit });
  expect(result.status).toBe(0);
  expect(JSON.parse(await Bun.file(lock).text())).toEqual({
    formulae: { git: "2.0" },
    casks: {},
  });
  const calls = result.calls.split("\n");
  const refresh = calls.lastIndexOf("brew info --json=v2 --formula git");
  expect(refresh).toBeGreaterThan(
    calls.indexOf("brew upgrade --formula --yes"),
  );
  expect(refresh).toBeGreaterThan(
    calls.indexOf("brew upgrade --cask --greedy --yes"),
  );
  expect(refresh).toBeLessThan(
    calls.indexOf("brew info --json=v2 --installed"),
  );
});
