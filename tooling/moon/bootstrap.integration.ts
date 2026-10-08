import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { expect, test } from "bun:test";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const root = join(import.meta.dir, "../..");
const pin = readFileSync(join(root, ".prototools"), "utf8");
const expectedVersion = z
  .object({ moon: z.string() })
  .parse(Bun.TOML.parse(pin)).moon;

type BootstrapFixture = Readonly<{
  directory: string;
  checkout: string;
  home: string;
  temporary: string;
  binary: string;
}>;

function checkoutFixture(
  pinContent: string | undefined,
  existing: boolean,
): BootstrapFixture {
  const directory = mkdtempSync(join(tmpdir(), "moon-bootstrap-smoke-"));
  const checkout = join(directory, "checkout");
  const home = join(directory, "home");
  const temporary = join(directory, "temporary");
  mkdirSync(join(checkout, "tooling"), { recursive: true });
  mkdirSync(join(home, ".moon/bin"), { recursive: true });
  mkdirSync(temporary);
  copyFileSync(
    join(root, "tooling/install-moon"),
    join(checkout, "tooling/install-moon"),
  );
  if (pinContent !== undefined) {
    writeFileSync(join(checkout, ".prototools"), pinContent);
  }
  writeFileSync(join(directory, ".prototools"), 'moon = "latest"');
  writeFileSync(join(home, ".prototools"), 'moon = "latest"');
  writeFileSync(join(checkout, ".prototools.foreign"), 'moon = "latest"');
  const binary = join(home, ".moon/bin/moon");
  if (existing) {
    writeFileSync(binary, '#!/bin/sh\necho "moon stale"\n', { mode: 0o755 });
  }
  return { directory, checkout, home, temporary, binary };
}

function verifyVersion(binary: string): void {
  const version = Bun.spawnSync([binary, "--version"]);
  expect(version.exitCode).toBe(0);
  expect(version.stdout.toString().trim()).toBe(`moon ${expectedVersion}`);
  process.stdout.write(
    `${process.platform}/${process.arch}: ${version.stdout.toString()}`,
  );
}

function bootstrap(pinContent: string | undefined, existing = false): void {
  const { directory, checkout, home, temporary, binary } = checkoutFixture(
    pinContent,
    existing,
  );
  try {
    const result = Bun.spawnSync([join(checkout, "tooling/install-moon")], {
      cwd: directory,
      env: {
        HOME: home,
        TMPDIR: temporary,
        PATH: "/usr/bin:/bin:/usr/sbin:/sbin",
        PROTO_MOON_VERSION: "latest",
        PROTO_ENV: "foreign",
        PROTO_OFFLINE: "true",
      },
    });
    if (pinContent === pin) {
      expect(result.stderr.toString()).not.toContain("error");
      expect(result.exitCode).toBe(0);
      verifyVersion(binary);
    } else {
      expect(result.exitCode).not.toBe(0);
      expect(result.stderr.toString()).toMatch(/error/iu);
      expect(existsSync(binary)).toBeFalse();
    }
    expect(readdirSync(temporary)).toEqual([]);
    expect(existsSync(join(home, ".proto"))).toBeFalse();
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("a fresh checkout executes its canonical Moon pin with only macOS system tools", () => {
  bootstrap(pin);
});

test("an existing checkout replaces a stale Moon binary with its canonical pin", () => {
  bootstrap(pin, true);
});

test.each([undefined, "", 'moon = "not-a-version"', 'moon = "'])(
  "refuses absent or invalid local Moon configuration: %s",
  (invalidPin) => {
    bootstrap(invalidPin);
  },
);
