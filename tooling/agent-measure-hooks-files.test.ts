import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  linkSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { deployAgentMeasureHooks } from "./agent-measure-hooks.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: string[] = [];

afterEach(() => {
  for (const directory of fixtures.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

function fixtureHome(): string {
  const directory = mkdtempSync(join(tmpdir(), "measure-hooks-"));
  fixtures.push(directory);
  return directory;
}

function writeSettings(home: string, relative: string, value: unknown): void {
  const destination = join(home, relative);
  mkdirSync(join(destination, ".."), { recursive: true });
  writeFileSync(destination, JSON.stringify(value));
}

test.each(["invalid JSON", "[]", '{"hooks":{"Stop":42}}'])(
  "preserves every file when a configuration is malformed: %s",
  (malformed) => {
    const home = fixtureHome();
    writeSettings(home, ".claude/settings.json", { model: "keep-model" });
    const original = readFileSync(join(home, ".claude/settings.json"), "utf8");
    mkdirSync(join(home, ".codex"));
    writeFileSync(join(home, ".codex/hooks.json"), malformed);

    expect(() => {
      deployAgentMeasureHooks(home);
    }).toThrow();

    expect(readFileSync(join(home, ".claude/settings.json"), "utf8")).toBe(
      original,
    );
    expect(readFileSync(join(home, ".codex/hooks.json"), "utf8")).toBe(
      malformed,
    );
    expect(existsSync(join(home, ".cursor/hooks.json"))).toBe(false);
  },
);

test("refuses a symbolic destination without modifying its target", () => {
  const home = fixtureHome();
  const target = join(home, "personal-settings.json");
  const destination = join(home, ".claude/settings.json");
  writeFileSync(target, '{"model":"keep-model"}');
  mkdirSync(join(home, ".claude"));
  symlinkSync(target, destination);

  expect(() => {
    deployAgentMeasureHooks(home);
  }).toThrow();

  expect(readlinkSync(destination)).toBe(target);
  expect(readFileSync(target, "utf8")).toBe('{"model":"keep-model"}');
});

test("refuses a symbolic agent directory before writing other agents", () => {
  const home = fixtureHome();
  const target = fixtureHome();
  writeSettings(home, ".claude/settings.json", { model: "keep-model" });
  const before = readFileSync(join(home, ".claude/settings.json"), "utf8");
  symlinkSync(target, join(home, ".cursor"));

  expect(() => {
    deployAgentMeasureHooks(home);
  }).toThrow();

  expect(readFileSync(join(home, ".claude/settings.json"), "utf8")).toBe(
    before,
  );
  expect(existsSync(join(home, ".codex/hooks.json"))).toBe(false);
  expect(existsSync(join(target, "hooks.json"))).toBe(false);
});

test("preserves another file sharing the original settings inode", () => {
  const home = fixtureHome();
  const target = join(home, "personal-settings.json");
  writeFileSync(target, '{"model":"keep-model"}');
  mkdirSync(join(home, ".claude"));
  linkSync(target, join(home, ".claude/settings.json"));

  deployAgentMeasureHooks(home);

  expect(readFileSync(target, "utf8")).toBe('{"model":"keep-model"}');
});

test("the public command reports invalid input without exposing its contents", () => {
  const home = fixtureHome();
  const privateValue = "SYNTHETIC_PRIVATE_CONTENT";
  mkdirSync(join(home, ".claude"));
  writeFileSync(
    join(home, ".claude/settings.json"),
    `{"value":"${privateValue}"`,
  );

  const result = Bun.spawnSync([
    process.execPath,
    "--config=/dev/null",
    "--no-env-file",
    join(import.meta.dir, "agent-measure-hooks.ts"),
    home,
  ]);

  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("Invalid hook settings");
  expect(result.stderr.toString()).not.toContain(privateValue);
  expect(result.stdout.toString()).toBe("");
});

test("creates private files and preserves existing permissions", () => {
  const home = fixtureHome();
  const existingMode = 0o640;
  const privateMode = 0o600;
  const permissionMask = 0o777;
  const restrictiveMask = 0o077;
  writeSettings(home, ".claude/settings.json", { model: "keep-model" });
  chmodSync(join(home, ".claude/settings.json"), existingMode);

  const previousMask = process.umask(restrictiveMask);
  try {
    deployAgentMeasureHooks(home);

    expect(
      lstatSync(join(home, ".claude/settings.json")).mode & permissionMask,
    ).toBe(existingMode);
    expect(
      lstatSync(join(home, ".codex/hooks.json")).mode & permissionMask,
    ).toBe(privateMode);
  } finally {
    process.umask(previousMask);
  }
});
