import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const configuration = z
  .object({ tasks: z.record(z.string(), z.unknown()) })
  .parse(
    Bun.YAML.parse(
      await Bun.file(new URL("../../moon.yml", import.meta.url)).text(),
    ),
  );

const downloadFailureExitCode = 22;
const installerFailureExitCode = 42;

function runInstaller(
  installer: string,
  downloadStatus: number,
): Readonly<{ exitCode: number; stdout: string }> {
  const task = configuration.tasks.homebrew;
  expect(task).toBeDefined();
  const { script } = z.object({ script: z.string() }).parse(task);
  const directory = mkdtempSync(join(tmpdir(), "homebrew-install-test-"));

  try {
    writeFileSync(
      join(directory, "curl"),
      '#!/bin/sh\nprintf "%s" "$HOMEBREW_TEST_INSTALLER"\nexit "$HOMEBREW_TEST_DOWNLOAD_STATUS"\n',
      { mode: 0o755 },
    );

    const result = Bun.spawnSync(["/bin/bash", "-c", script], {
      env: {
        PATH: directory,
        HOMEBREW_TEST_INSTALLER: installer,
        HOMEBREW_TEST_DOWNLOAD_STATUS: String(downloadStatus),
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
