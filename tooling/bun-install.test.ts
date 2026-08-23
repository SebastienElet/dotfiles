import { afterEach, beforeEach, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

let testRoot: string;

beforeEach(() => {
  testRoot = mkdtempSync(join(tmpdir(), "bun-install-"));
});

afterEach(() => {
  rmSync(testRoot, { force: true, recursive: true });
});

function executable(path: string, body: string): void {
  writeFileSync(path, `#!/bin/sh\n${body}\n`);
  chmodSync(path, 0o755);
}

test("repairs an installed Bun at the wrong version and then remains stable", async () => {
  const fakeBin = join(testRoot, "fake-bin");
  const home = join(testRoot, "home");
  const bunBin = join(home, ".bun", "bin", "bun");
  const downloadMarker = join(testRoot, "downloaded");
  const replacement = join(testRoot, "replacement-bun");

  mkdirSync(join(home, ".bun", "bin"), { recursive: true });
  mkdirSync(fakeBin);
  executable(bunBin, "printf '0.0.0\\n'");
  executable(join(fakeBin, "uname"), "printf 'arm64\\n'");
  executable(
    join(fakeBin, "curl"),
    'while [ "$#" -gt 0 ]; do if [ "$1" = "-o" ]; then shift; archive=$1; fi; shift; done\nprintf archive > "$archive"\nprintf downloaded > "$DOWNLOAD_MARKER"',
  );
  executable(join(fakeBin, "shasum"), "cat >/dev/null");
  executable(replacement, "printf '1.4.0\\n'");
  executable(
    join(fakeBin, "unzip"),
    'while [ "$#" -gt 0 ]; do if [ "$1" = "-d" ]; then shift; destination=$1; fi; shift; done\nmkdir -p "$destination/bun-darwin-aarch64"\ncp "$REPLACEMENT_BUN" "$destination/bun-darwin-aarch64/bun"',
  );

  const environment = {
    ...process.env,
    DOWNLOAD_MARKER: downloadMarker,
    HOME: home,
    PATH: `${fakeBin}:${process.env.PATH ?? ""}`,
    REPLACEMENT_BUN: replacement,
  };
  const command = [
    "make",
    "-f",
    join(import.meta.dir, "..", "Makefile"),
    "bun",
  ];

  const repaired = Bun.spawnSync(command, { env: environment });
  expect(repaired.exitCode).toBe(0);
  expect(readFileSync(bunBin, "utf8")).toContain("1.4.0");
  expect(existsSync(downloadMarker)).toBe(true);

  rmSync(downloadMarker);
  const stable = Bun.spawnSync(command, { env: environment });
  expect(stable.exitCode).toBe(0);
  expect(existsSync(downloadMarker)).toBe(false);
});
