import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { tmpdir } from "node:os";

const makefile = fileURLToPath(new URL("../../Makefile", import.meta.url));
const downloadFailureExitCode = 22;

function bootstrapMoon(
  installer: string,
  downloadStatus: number,
): Readonly<{ exitCode: number; stdout: string }> {
  const directory = mkdtempSync(join(tmpdir(), "moon-bootstrap-test-"));

  try {
    writeFileSync(
      join(directory, "curl"),
      '#!/bin/sh\nprintf "%s" "$MOON_TEST_INSTALLER"\nexit "$MOON_TEST_DOWNLOAD_STATUS"\n',
      { mode: 0o755 },
    );
    const result = Bun.spawnSync(
      ["/usr/bin/make", "--no-print-directory", "-f", makefile, "moon"],
      {
        env: {
          HOME: directory,
          PATH: `${directory}:/usr/bin:/bin`,
          MOON_TEST_INSTALLER: installer,
          MOON_TEST_DOWNLOAD_STATUS: String(downloadStatus),
        },
        stdout: "pipe",
        stderr: "pipe",
      },
    );

    return { exitCode: result.exitCode, stdout: result.stdout.toString() };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("runs the downloaded Moon installer", () => {
  const result = bootstrapMoon('printf "installer executed"', 0);
  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain("installer executed");
});

test("does not execute partial Moon installer content after download failure", () => {
  const result = bootstrapMoon(
    'printf "installer executed"',
    downloadFailureExitCode,
  );
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).not.toContain("installer executed");
});

test("propagates a Moon installer failure", () => {
  expect(bootstrapMoon("exit 1", 0).exitCode).not.toBe(0);
});
