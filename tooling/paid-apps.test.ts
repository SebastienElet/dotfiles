import { afterEach, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const makefile = join(import.meta.dir, "..", "Makefile");
const fixtures: string[] = [];
type PaidApp = Readonly<{ bundle: string; target: string }>;
const paidApps: PaidApp[] = [
  { bundle: "Things3.app", target: "things-3" },
  { bundle: "DaisyDisk.app", target: "daisydisk" },
];

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test.each(paidApps)(
  "$target: an explicit paid-app skip succeeds without $bundle",
  ({ bundle, target }) => {
    const fixture = createFixture();

    const result = runTarget(fixture, target, true);

    expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
    expect(() => readFileSync(join(fixture.apps, bundle))).toThrow();
  },
);

test.each(paidApps)(
  "$target: fails when Bundle succeeds without producing $bundle",
  ({ target }) => {
    const fixture = createFixture();

    const result = runTarget(fixture, target, false);

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("Homebrew Bundle did not install");
  },
);

test.each(paidApps)(
  "$target: accepts an existing $bundle without changing it",
  ({ bundle, target }) => {
    const fixture = createFixture();
    const marker = join(fixture.apps, bundle, "existing-data");
    mkdirSync(join(fixture.apps, bundle));
    writeFileSync(marker, "preserve");

    const result = runTarget(fixture, target, false);

    expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
    expect(readFileSync(marker, "utf8")).toBe("preserve");
  },
);

function createFixture(): Readonly<{
  apps: string;
  brewBin: string;
  root: string;
  voltaBin: string;
}> {
  const root = mkdtempSync(join(tmpdir(), "dotfiles-paid-apps-"));
  const apps = join(root, "Applications");
  const brewBin = join(root, "homebrew", "bin");
  const voltaBin = join(root, "volta", "bin");
  fixtures.push(root);
  mkdirSync(apps);
  mkdirSync(brewBin, { recursive: true });
  mkdirSync(voltaBin, { recursive: true });
  writeFileSync(join(brewBin, "bun"), "");
  writeFileSync(join(brewBin, "volta"), "");
  writeFileSync(join(voltaBin, "node"), "");
  writeFileSync(join(voltaBin, "npm"), "");
  writeFileSync(join(voltaBin, "thangs"), "");
  return { apps, brewBin, root, voltaBin };
}

function runTarget(
  fixture: ReturnType<typeof createFixture>,
  target: string,
  skipPaidApps: boolean,
): Readonly<{ exitCode: number; stderr: string; stdout: string }> {
  const result = Bun.spawnSync(
    [
      "make",
      "--no-print-directory",
      "-f",
      makefile,
      target,
      `APP_BIN=${fixture.apps}`,
      `BREW_BIN=${fixture.brewBin}`,
      `VOLTA_BIN=${fixture.voltaBin}`,
      `SKIP_PAID_APPS=${skipPaidApps ? "1" : "0"}`,
    ],
    { stderr: "pipe", stdout: "pipe" },
  );
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}
