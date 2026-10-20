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

type CheckoutState =
  | "fresh"
  | "stale-moon"
  | "missing-lock"
  | "invalid-integrity";

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

function writeMoonLock(manifest: string, state: CheckoutState): void {
  if (state !== "missing-lock") {
    const lock = readFileSync(
      join(root, manifestDirectory, "bun.lock"),
      "utf8",
    );
    writeFileSync(
      join(manifest, "bun.lock"),
      state === "invalid-integrity"
        ? lock.replace(/sha512-./u, "sha512-A")
        : lock,
    );
  }
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
  writeMoonLock(manifest, state);
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
  overrides: Readonly<Record<string, string>> = {},
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
        ...overrides,
      },
    });
    if (pinContent === pin && (state === "fresh" || state === "stale-moon")) {
      expect(result.stderr.toString()).not.toContain("error");
      expect(result.exitCode).toBe(0);
      verifyVersion(binary);
    } else {
      expect(result.exitCode).not.toBe(0);
      expect(result.stderr.toString()).not.toBe("");
      expect(existsSync(binary)).toBeFalse();
      if (state === "invalid-integrity") {
        expect(result.stderr.toString()).toContain("IntegrityCheckFailed");
      }
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

test("loads the committed lock despite a caller override", () => {
  bootstrap(pin.replace(expectedVersion, "2.5.2"), "fresh", {
    BUN_CONFIG_SKIP_LOAD_LOCKFILE: "1",
  });
});

test.each([{ BUN_CONFIG_NO_VERIFY: "0" }, { BUN_OPTIONS: "--no-verify" }])(
  "verifies package integrity despite caller overrides: %j",
  (overrides: Readonly<Record<string, string>>) => {
    bootstrap(pin, "invalid-integrity", overrides);
  },
);
