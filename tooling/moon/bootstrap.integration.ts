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
const manifestDirectory = ".github/actions/setup-moon";
const pin = readFileSync(join(root, manifestDirectory, "package.json"), "utf8");
const expectedVersion = z
  .object({ dependencies: z.object({ "@moonrepo/cli": z.string() }) })
  .parse(JSON.parse(pin)).dependencies["@moonrepo/cli"];

type CheckoutState = "fresh" | "stale-moon" | "missing-lock";

type BootstrapFixture = Readonly<{
  directory: string;
  checkout: string;
  home: string;
  temporary: string;
  binary: string;
}>;

function writeForeignConfiguration(directory: string, home: string): void {
  writeFileSync(join(home, ".bunfig.toml"), "invalid = [");
  const foreignConfig = join(directory, "foreign-config");
  mkdirSync(foreignConfig);
  writeFileSync(join(foreignConfig, ".bunfig.toml"), "invalid = [");
  writeFileSync(join(directory, ".prototools"), 'moon = "latest"');
  writeFileSync(join(home, ".prototools"), 'moon = "latest"');
}

function checkoutFixture(
  pinContent: string | undefined,
  state: CheckoutState,
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
  const manifest = join(checkout, manifestDirectory);
  mkdirSync(manifest, { recursive: true });
  if (state !== "missing-lock") {
    copyFileSync(
      join(root, manifestDirectory, "bun.lock"),
      join(manifest, "bun.lock"),
    );
  }
  if (pinContent !== undefined) {
    writeFileSync(join(manifest, "package.json"), pinContent);
  }
  writeForeignConfiguration(directory, home);
  writeFileSync(join(checkout, ".prototools.foreign"), 'moon = "latest"');
  const binary = join(home, ".moon/bin/moon");
  if (state === "stale-moon") {
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

function bootstrap(
  pinContent: string | undefined,
  state: CheckoutState = "fresh",
): void {
  const { directory, checkout, home, temporary, binary } = checkoutFixture(
    pinContent,
    state,
  );
  try {
    const result = Bun.spawnSync([join(checkout, "tooling/install-moon")], {
      cwd: directory,
      env: {
        HOME: home,
        TMPDIR: temporary,
        XDG_CONFIG_HOME: join(directory, "foreign-config"),
        PATH: "/usr/bin:/bin:/usr/sbin:/sbin",
        PROTO_MOON_VERSION: "latest",
        PROTO_ENV: "foreign",
        PROTO_OFFLINE: "true",
        PROTO_CLI_INSTALL_DIR: join(directory, "foreign-proto"),
        CARGO_DIST_FORCE_INSTALL_DIR: join(directory, "foreign-cargo"),
      },
    });
    if (pinContent === pin && state !== "missing-lock") {
      expect(result.stderr.toString()).not.toContain("error");
      expect(result.exitCode).toBe(0);
      verifyVersion(binary);
    } else {
      expect(result.exitCode).not.toBe(0);
      expect(result.stderr.toString()).not.toBe("");
      expect(existsSync(binary)).toBeFalse();
    }
    expect(readdirSync(temporary)).toEqual([]);
    expect(existsSync(join(home, ".proto"))).toBeFalse();
    expect(existsSync(join(home, ".bun"))).toBeFalse();
    expect(existsSync(join(directory, "foreign-proto"))).toBeFalse();
    expect(existsSync(join(directory, "foreign-cargo"))).toBeFalse();
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("a fresh checkout executes its canonical Moon pin with only macOS system tools", () => {
  bootstrap(pin);
});

test("an existing checkout replaces a stale Moon binary with its canonical pin", () => {
  bootstrap(pin, "stale-moon");
});

test.each([undefined, "", "{", pin.replace(expectedVersion, "0.0.0")])(
  "refuses absent or invalid local Moon configuration: %s",
  (invalidPin) => {
    bootstrap(invalidPin);
  },
);

test("refuses a checkout without its Moon lockfile", () => {
  bootstrap(pin, "missing-lock");
});
