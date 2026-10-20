import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { expect, test } from "bun:test";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { tmpdir } from "node:os";

const bootstrap = fileURLToPath(new URL("../install-moon", import.meta.url));
const failureExitCode = 42;
const downloadFailureExitCode = 22;
const downloadTimeoutExitCode = 28;
const selectedVersion = "moon 2.5.3";

function writeBootstrapCommands(directory: string): void {
  const commands = {
    curl: `printf 'untrusted download' > "$7"; exit "$MOON_TEST_DOWNLOAD_STATUS"`,
    shasum: `cat >/dev/null; exit "$MOON_TEST_CHECKSUM_STATUS"`,
    unzip: `
      [ "$MOON_TEST_UNZIP_STATUS" = 0 ] || exit "$MOON_TEST_UNZIP_STATUS"
      mkdir -p "$4/bun-darwin-aarch64"
      cp "$MOON_TEST_BUN" "$4/bun-darwin-aarch64/bun"
    `,
    bun: `
      [ "$MOON_TEST_BUN_STATUS" = 0 ] || exit "$MOON_TEST_BUN_STATUS"
      [ "$1" = install ] || exit 89
      [ -f package.json ] && [ -f bun.lock ] || exit 89
      mkdir -p node_modules/@moonrepo/core-macos-arm64
      cp "$MOON_TEST_BINARY" node_modules/@moonrepo/core-macos-arm64/moon
    `,
    moon: `printf '%s\\n' '${selectedVersion}'; exit "$MOON_TEST_VERSION_STATUS"`,
    install: `[ "$MOON_TEST_COPY_STATUS" = 0 ] || exit "$MOON_TEST_COPY_STATUS"; exec /usr/bin/install "$@"`,
    uname: `case "$1" in -s) echo Darwin ;; -m) echo "$MOON_TEST_ARCH" ;; esac`,
  };
  for (const [name, body] of Object.entries(commands)) {
    writeFileSync(join(directory, name), `#!/bin/sh\n${body}\n`, {
      mode: 0o755,
    });
  }
}

type BootstrapFailures = Readonly<{
  download?: number;
  checksum?: number;
  unzip?: number;
  bun?: number;
  copy?: number;
  version?: number;
  realChecksum?: boolean;
  arch?: string;
}>;

function bootstrapEnvironment(
  directory: string,
  failures: BootstrapFailures,
): NodeJS.ProcessEnv {
  return {
    HOME: directory,
    TMPDIR: join(directory, "temporary"),
    PATH: `${directory}:/usr/bin:/bin`,
    MOON_TEST_ARCH: failures.arch ?? "arm64",
    MOON_TEST_DOWNLOAD_STATUS: String(failures.download ?? 0),
    MOON_TEST_CHECKSUM_STATUS: String(failures.checksum ?? 0),
    MOON_TEST_UNZIP_STATUS: String(failures.unzip ?? 0),
    MOON_TEST_BUN_STATUS: String(failures.bun ?? 0),
    MOON_TEST_BUN: join(directory, "bun"),
    MOON_TEST_BINARY: join(directory, "moon"),
    MOON_TEST_COPY_STATUS: String(failures.copy ?? 0),
    MOON_TEST_VERSION_STATUS: String(failures.version ?? 0),
  };
}

function bootstrapMoon(failures: BootstrapFailures = {}): Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
  installed: boolean;
}> {
  const directory = mkdtempSync(join(tmpdir(), "moon-bootstrap-test-"));
  try {
    writeBootstrapCommands(directory);
    if (failures.realChecksum === true) {
      rmSync(join(directory, "shasum"));
    }
    const temporary = join(directory, "temporary");
    mkdirSync(temporary);
    const result = Bun.spawnSync([bootstrap], {
      cwd: directory,
      env: bootstrapEnvironment(directory, failures),
      stdout: "pipe",
      stderr: "pipe",
    });
    const installed = existsSync(join(directory, ".moon/bin/moon"));
    expect(readdirSync(temporary)).toEqual([]);
    return {
      exitCode: result.exitCode,
      stdout: result.stdout.toString(),
      stderr: result.stderr.toString(),
      installed,
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("installs and executes the locked Moon binary through Bun", () => {
  const result = bootstrapMoon();
  expect(result.stderr).toBe("");
  expect(result.exitCode).toBe(0);
  expect(result.installed).toBeTrue();
  expect(result.stdout).toContain(selectedVersion);
});

test.each([downloadFailureExitCode, downloadTimeoutExitCode])(
  "reports download failure %s before extracting or executing Bun",
  (download) => {
    const result = bootstrapMoon({ download });
    expect(result.exitCode).toBe(download);
    expect(result.stderr).toContain("Error: unable to download Bun");
    expect(result.installed).toBeFalse();
  },
);

test("rejects a corrupt download with the real SHA-256 checker", () => {
  const result = bootstrapMoon({ realChecksum: true });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Error: Bun checksum verification failed");
  expect(result.installed).toBeFalse();
});

test.each(["checksum", "unzip", "bun", "copy", "version"] as const)(
  "propagates failure while performing %s",
  (stage) => {
    const result = bootstrapMoon({ [stage]: failureExitCode });
    expect(result.exitCode).toBe(failureExitCode);
    expect(result.installed).toBe(stage === "version");
  },
);

test("refuses unsupported platforms before installing Moon", () => {
  const result = bootstrapMoon({ arch: "unsupported" });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Error: unsupported bootstrap platform");
  expect(result.installed).toBeFalse();
});
