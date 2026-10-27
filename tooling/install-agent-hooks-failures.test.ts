import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  fileIdentity,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  hooksFixture,
  runHookInstaller,
  writeHookManifest,
} from "./install-agent-hooks-test-support.ts";
import {
  linkSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";

afterEach(cleanupDeploymentFixtures);

test.each(["malformed", "unknown-kind", "duplicate", "cursor-kind"])(
  "refuses %s manifest before changing existing settings",
  (kind) => {
    const { fixture, destination } = hooksFixture("cursor");
    writeFileSync(destination, '{"version":1,"hooks":{}}');
    const before = fileIdentity(destination);
    if (kind === "malformed") {
      writeFileSync(join(fixture.repository, "home/.arnes.yaml"), "hooks: [");
    } else if (kind === "unknown-kind") {
      writeFileSync(
        join(fixture.repository, "home/.arnes.yaml"),
        '{"version":1,"hooks":[{"id":"unknown","installations":[{"agent":"cursor","scope":"user"}]}]}',
      );
    } else {
      writeHookManifest(
        fixture,
        "cursor",
        kind === "duplicate" ? ["measurement", "measurement"] : ["handoff"],
      );
    }
    const result = runHookInstaller(fixture, "cursor");
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).not.toBe("");
    expect(fileIdentity(destination)).toEqual(before);
  },
);

test.each([
  "not json",
  "[]",
  '{"hooks":null}',
  '{"hooks":{"Stop":null}}',
  '{"hooks":{"Stop":[{"hooks":null}]}}',
])("rejects malformed hook configuration without rewriting it: %s", (input) => {
  const { fixture, destination } = hooksFixture("claude");
  writeFileSync(destination, input);
  const before = fileIdentity(destination);
  expect(runHookInstaller(fixture, "claude").exitCode).not.toBe(0);
  expect(fileIdentity(destination)).toEqual(before);
  expect(readdirSync(dirname(destination))).toEqual(["settings.json"]);
});

test.each(["symlink", "hardlink", "directory"])(
  "refuses a %s hook file without changing its target",
  (kind) => {
    const { fixture, destination } = hooksFixture("codex");
    const foreign = join(fixture.root, "personal.json");
    writeFileSync(foreign, "{}");
    if (kind === "symlink") {
      symlinkSync(foreign, destination);
    } else if (kind === "hardlink") {
      linkSync(foreign, destination);
    } else {
      mkdirSync(destination);
    }
    expect(runHookInstaller(fixture, "codex").exitCode).not.toBe(0);
    expect(readFileSync(foreign, "utf8")).toBe("{}");
  },
);

test("refuses a symbolic agent directory before creating files in its target", () => {
  const { fixture, destination } = hooksFixture("codex");
  const foreign = join(fixture.root, "personal-agent");
  mkdirSync(foreign);
  rmSync(dirname(destination), { recursive: true });
  symlinkSync(foreign, dirname(destination));
  expect(runHookInstaller(fixture, "codex").exitCode).not.toBe(0);
  expect(readdirSync(foreign)).toEqual([]);
});

test("refuses oversized input and releases its lock", () => {
  const { fixture, destination } = hooksFixture("codex");
  const oversizedBytes = 1_048_577;
  writeFileSync(destination, " ".repeat(oversizedBytes));
  const before = fileIdentity(destination);
  expect(runHookInstaller(fixture, "codex").exitCode).not.toBe(0);
  expect(fileIdentity(destination)).toEqual(before);
  expect(readdirSync(dirname(destination))).toEqual(["hooks.json"]);
});

test("a concurrent or abandoned installer lock refuses mutation", () => {
  const { fixture, destination } = hooksFixture("codex");
  writeFileSync(destination, "{}");
  mkdirSync(`${destination}.dotfiles-lock`);
  const before = fileIdentity(destination);
  expect(runHookInstaller(fixture, "codex").exitCode).not.toBe(0);
  expect(fileIdentity(destination)).toEqual(before);
});
