import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const installerPath = join(import.meta.dir, "../install-homebrew");

const downloadFailureExitCode = 22;
const installerFailureExitCode = 42;

function runInstaller(
  installer: string,
  downloadStatus: number,
  checksumStatus = 0,
): Readonly<{ exitCode: number; stdout: string }> {
  const directory = mkdtempSync(join(tmpdir(), "homebrew-install-test-"));

  try {
    writeFileSync(
      join(directory, "curl"),
      '#!/bin/sh\nwhile [ "$1" != -o ]; do shift; done\nprintf "%s" "$HOMEBREW_TEST_INSTALLER" > "$2"\nexit "$HOMEBREW_TEST_DOWNLOAD_STATUS"\n',
      { mode: 0o755 },
    );
    writeFileSync(
      join(directory, "shasum"),
      '#!/bin/sh\ncat > /dev/null\nexit "$HOMEBREW_TEST_CHECKSUM_STATUS"\n',
      { mode: 0o755 },
    );
    for (const command of ["mktemp", "rm"]) {
      symlinkSync(Bun.which(command) ?? command, join(directory, command));
    }

    const result = Bun.spawnSync(["/bin/bash", installerPath], {
      env: {
        PATH: directory,
        HOMEBREW_TEST_INSTALLER: installer,
        HOMEBREW_TEST_DOWNLOAD_STATUS: String(downloadStatus),
        HOMEBREW_TEST_CHECKSUM_STATUS: String(checksumStatus),
      },
      stdout: "pipe",
      stderr: "pipe",
    });

    return { exitCode: result.exitCode, stdout: result.stdout.toString() };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("executes the downloaded Homebrew installer", () => {
  expect(runInstaller('printf "installer executed"', 0)).toEqual({
    exitCode: 0,
    stdout: "installer executed",
  });
});

test("does not execute partial installer content when the download fails", () => {
  expect(
    runInstaller('printf "installer executed"', downloadFailureExitCode),
  ).toEqual({
    exitCode: downloadFailureExitCode,
    stdout: "",
  });
});

test("propagates a Homebrew installer failure", () => {
  expect(runInstaller(`exit ${installerFailureExitCode}`, 0).exitCode).toBe(
    installerFailureExitCode,
  );
});

test("does not execute an installer whose checksum differs from the pin", () => {
  expect(runInstaller('printf "installer executed"', 0, 1)).toEqual({
    exitCode: 1,
    stdout: "",
  });
});
