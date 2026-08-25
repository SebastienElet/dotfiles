import {
  type CommandLinkFileSystem,
  retireCommandLink,
} from "./retire-command-link.ts";
import { afterEach, expect, test } from "bun:test";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  renameSync,
  rmSync,
  rmdirSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const roots: string[] = [];
const QUARANTINED_READ = 2;
const fileSystem: CommandLinkFileSystem = {
  lstat: lstatSync,
  makeTemporaryDirectory: mkdtempSync,
  readlink: readlinkSync,
  removeDirectory: rmdirSync,
  rename: renameSync,
  symlink: symlinkSync,
};

afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test("rejects a relative command-line path", () => {
  const entryPoint = join(import.meta.dir, "retire-command-link");
  const result = Bun.spawnSync([entryPoint, "relative", "/destination"], {
    stderr: "pipe",
    stdout: "pipe",
  });

  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("paths must be absolute");
});

test("restores an unexpected link substituted during quarantine", () => {
  const root = createRoot();
  const destination = join(root, "command");
  const expected = join(root, "expected");
  const unexpected = join(root, "unexpected");
  symlinkSync(expected, destination);
  let raced = false;
  const racingFileSystem: CommandLinkFileSystem = {
    ...fileSystem,
    rename(source, target) {
      if (!raced) {
        raced = true;
        unlinkSync(source);
        symlinkSync(unexpected, source);
      }
      renameSync(source, target);
    },
  };

  expect(() =>
    retireCommandLink(expected, destination, racingFileSystem),
  ).toThrow("changed during retirement and was restored");
  expect(readlinkSync(destination)).toBe(unexpected);
  const quarantine = readdirSync(root).find((entry) =>
    entry.startsWith(".retire-command-link-"),
  );
  expect(quarantine).toBeDefined();
  expect(readlinkSync(join(root, quarantine ?? "", "entry"))).toBe(unexpected);
});

test("does not delete an entry substituted after quarantine validation", () => {
  const root = createRoot();
  const destination = join(root, "command");
  const expected = join(root, "expected");
  symlinkSync(expected, destination);
  let readCount = 0;
  const racingFileSystem: CommandLinkFileSystem = {
    ...fileSystem,
    readlink(path) {
      const target = readlinkSync(path);
      readCount += 1;
      if (readCount === QUARANTINED_READ) {
        unlinkSync(path);
        writeFileSync(path, "valuable-data");
      }
      return target;
    },
  };

  expect(retireCommandLink(expected, destination, racingFileSystem)).toBe(
    "removed",
  );
  const quarantine = readdirSync(root).find((entry) =>
    entry.startsWith(".retire-command-link-"),
  );
  expect(quarantine).toBeDefined();
  expect(readFileSync(join(root, quarantine ?? "", "entry"), "utf8")).toBe(
    "valuable-data",
  );
});

test("surfaces destination probe failures", () => {
  const root = createRoot();
  const destination = join(root, "command");
  const failingFileSystem: CommandLinkFileSystem = {
    ...fileSystem,
    lstat() {
      throw new Error("probe failed");
    },
  };

  expect(() =>
    retireCommandLink(join(root, "expected"), destination, failingFileSystem),
  ).toThrow(`could not inspect ${destination}`);
});

function createRoot(): string {
  const root = mkdtempSync(join(tmpdir(), "retire-command-link-"));
  roots.push(root);
  mkdirSync(root, { recursive: true });
  return root;
}
