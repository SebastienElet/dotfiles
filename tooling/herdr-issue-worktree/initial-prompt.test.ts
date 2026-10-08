import { afterEach, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";

const directories: string[] = [];
const longRequestRepeats = 4000;
const nativeCanonicalInputLimit = 1023;
const dataSeparatorLength = 2;
const permissionMask = 0o777;
const privateFileMode = 0o600;
const privateDirectoryMode = 0o700;
const excessivePathRepeats = 100;
function directory(): string {
  const path = mkdtempSync(join(tmpdir(), "herdr-initial-prompt-"));
  directories.push(path);
  return path;
}
function encode(
  path: string,
  prompt: string,
): Readonly<{ exitCode: number; stdout: Buffer; stderr: Buffer }> {
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      join(import.meta.dir, "initial-prompt.ts"),
      path,
    ],
    { stdin: new Blob([prompt]), stdout: "pipe", stderr: "pipe" },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}
afterEach(() => {
  for (const path of directories.splice(0)) {
    rmSync(path, { recursive: true, force: true });
  }
});
test("encodes a short private-file reference with the complete nested request", () => {
  const root = directory();
  const prompt = JSON.stringify({
    nested: JSON.stringify({
      selected:
        "quotes ' \" \\ $(touch marker) `code`\n\u0000\u007F\u0085🐟\uD800",
    }),
    context: "完整 ".repeat(longRequestRepeats),
  });
  const result = encode(root, prompt);
  expect(result.exitCode).toBe(0);
  expect(result.stderr.toString()).toBe("");
  const argument = result.stdout.toString();
  expect(Buffer.byteLength(argument)).toBeLessThan(nativeCanonicalInputLimit);
  expect(argument).not.toContain("\n");
  const path: unknown = JSON.parse(
    Buffer.from(
      argument.slice(argument.lastIndexOf(": ") + dataSeparatorLength),
      "base64",
    ).toString("utf8"),
  );
  if (typeof path !== "string") {
    throw new TypeError("Expected an initial request path");
  }
  expect(JSON.parse(readFileSync(path, "utf8"))).toBe(prompt);
  expect(lstatSync(path).mode & permissionMask).toBe(privateFileMode);
  expect(lstatSync(dirname(path)).mode & permissionMask).toBe(
    privateDirectoryMode,
  );
});
test.each(["relative", `/${"long-path".repeat(excessivePathRepeats)}`])(
  "refuses unusable reference path %s before producing a launch argument",
  (path) => {
    const result = encode(path, "request");
    expect(result.exitCode).toBe(1);
    expect(result.stdout.toString()).toBe("");
    expect(result.stderr.length).toBeGreaterThan(0);
  },
);
test("refuses a symlink or non-directory without overwriting its target", () => {
  const root = directory();
  const target = join(root, "existing.json");
  writeFileSync(target, "preserve");
  const link = join(root, "linked");
  symlinkSync(root, link);
  for (const path of [target, link]) {
    const result = encode(path, "request");
    expect(result.exitCode).toBe(1);
    expect(result.stdout.toString()).toBe("");
    expect(result.stderr.toString()).toContain("real directory");
  }
  expect(readFileSync(target, "utf8")).toBe("preserve");
});
test("reports a missing directory or empty request without a launch argument", () => {
  const root = directory();
  for (const [path, prompt] of [
    [join(root, "missing"), "request"],
    [root, ""],
  ]) {
    const result = encode(path ?? "", prompt ?? "");
    expect(result.exitCode).toBe(1);
    expect(result.stdout.toString()).toBe("");
  }
});
