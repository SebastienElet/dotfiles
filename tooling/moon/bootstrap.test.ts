import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { tmpdir } from "node:os";

const bootstrap = fileURLToPath(new URL("../install-moon", import.meta.url));
const downloadFailureExitCode = 22;
const installerFailureExitCode = 42;
const selectedVersion = "moon 2.5.2";
const installer = `
[ "$PROTO_CLI_UNMANAGED_INSTALL" = "$PROTO_HOME/bin" ] || exit 88
mkdir -p "$PROTO_HOME/bin"
cp "$MOON_TEST_PROTO" "$PROTO_HOME/bin/proto"
`;

function writeProtoFixture(directory: string): void {
  writeFileSync(
    join(directory, "proto"),
    `#!/bin/sh
[ "$PROTO_MOON_VERSION" = '' ] || exit 89
[ "$PROTO_ENV" = '' ] || exit 89
[ "$PROTO_DETECT_STRATEGY" = only-prototools ] || exit 89
[ -f .prototools ] || exit 89
case "$*" in
  '--config-mode local run moon -- --version') exit "$MOON_TEST_PROTO_STATUS" ;;
  '--config-mode local bin moon') printf '%s' "$MOON_TEST_BINARY"; exit "$MOON_TEST_BIN_STATUS" ;;
  *) exit 89 ;;
esac
`,
    { mode: 0o755 },
  );
}

function writeBootstrapCommands(directory: string): void {
  writeFileSync(
    join(directory, "curl"),
    '#!/bin/sh\nprintf "%s" "$MOON_TEST_INSTALLER"\nexit "$MOON_TEST_DOWNLOAD_STATUS"\n',
    { mode: 0o755 },
  );
  writeFileSync(
    join(directory, "moon"),
    `#!/bin/sh\nprintf '%s\\n' '${selectedVersion}'\nexit "$MOON_TEST_VERSION_STATUS"\n`,
    { mode: 0o755 },
  );
  writeFileSync(
    join(directory, "install"),
    '#!/bin/sh\n[ "$MOON_TEST_COPY_STATUS" = 0 ] || exit "$MOON_TEST_COPY_STATUS"\nexec /usr/bin/install "$@"\n',
    { mode: 0o755 },
  );
}

function bootstrapMoon(
  downloadedInstaller: string,
  downloadStatus: number,
  failures: Readonly<{
    run?: number;
    bin?: number;
    copy?: number;
    version?: number;
  }> = {},
): Readonly<{ exitCode: number; stdout: string; stderr: string }> {
  const directory = mkdtempSync(join(tmpdir(), "moon-bootstrap-test-"));
  try {
    writeBootstrapCommands(directory);
    writeProtoFixture(directory);
    const result = Bun.spawnSync([bootstrap], {
      cwd: directory,
      env: {
        HOME: directory,
        PATH: `${directory}:/usr/bin:/bin`,
        PROTO_MOON_VERSION: "latest",
        PROTO_ENV: "foreign",
        MOON_TEST_INSTALLER: downloadedInstaller,
        MOON_TEST_DOWNLOAD_STATUS: String(downloadStatus),
        MOON_TEST_PROTO: join(directory, "proto"),
        MOON_TEST_BINARY: join(directory, "moon"),
        MOON_TEST_PROTO_STATUS: String(failures.run ?? 0),
        MOON_TEST_BIN_STATUS: String(failures.bin ?? 0),
        MOON_TEST_COPY_STATUS: String(failures.copy ?? 0),
        MOON_TEST_VERSION_STATUS: String(failures.version ?? 0),
      },
      stdout: "pipe",
      stderr: "pipe",
    });
    return {
      exitCode: result.exitCode,
      stdout: result.stdout.toString(),
      stderr: result.stderr.toString(),
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("installs and executes the binary selected by local proto configuration", () => {
  const result = bootstrapMoon(installer, 0);
  expect(result.stderr).toBe("");
  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain(selectedVersion);
});

test("does not execute partial installer content after download failure", () => {
  const result = bootstrapMoon(
    'printf "installer executed"',
    downloadFailureExitCode,
  );
  expect(result.exitCode).toBe(downloadFailureExitCode);
  expect(result.stdout).not.toContain("installer executed");
});

test("propagates a proto installer failure", () => {
  expect(bootstrapMoon(`exit ${installerFailureExitCode}`, 0).exitCode).toBe(
    installerFailureExitCode,
  );
});

test("stops before copying Moon when version resolution or installation fails", () => {
  const result = bootstrapMoon(installer, 0, { run: installerFailureExitCode });
  expect(result.exitCode).toBe(installerFailureExitCode);
  expect(result.stdout).not.toContain(selectedVersion);
});

test.each(["bin", "copy", "version"] as const)(
  "propagates failure while performing %s",
  (stage) => {
    const result = bootstrapMoon(installer, 0, {
      [stage]: installerFailureExitCode,
    });
    expect(result.exitCode).toBe(installerFailureExitCode);
  },
);
