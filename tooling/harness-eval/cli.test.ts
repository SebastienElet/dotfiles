import { afterEach, expect, test } from "bun:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { reportSchema } from "./report-schema.ts";
import { tmpdir } from "node:os";

const roots: string[] = [];
afterEach(() => {
  for (const path of roots.splice(0)) {
    rmSync(path, { recursive: true, force: true });
  }
});
const cli = resolve(import.meta.dir, "cli.ts");

function setup(): {
  root: string;
  marker: string;
  invoke: (args: readonly string[]) => {
    readonly exitCode: number;
    readonly stdout: Buffer;
  };
} {
  const root = mkdtempSync(join(tmpdir(), "harness-cli-test-"));
  roots.push(root);
  const bin = join(root, "bin");
  mkdirSync(bin);
  const marker = join(root, "llm-called");
  for (const name of ["codex", "claude", "curl"]) {
    writeFileSync(
      join(bin, name),
      `#!${process.execPath}\nawait Bun.write(${JSON.stringify(marker)}, "called"); process.exit(98);\n`,
      { mode: 0o755 },
    );
  }
  const env = { PATH: `${bin}:/usr/bin:/bin`, HOME: root };
  const invoke = (
    args: readonly string[],
  ): { readonly exitCode: number; readonly stdout: Buffer } =>
    Bun.spawnSync(
      [process.execPath, "--config=/dev/null", "--no-env-file", cli, ...args],
      { env },
    );
  return { root, marker, invoke };
}

test("public deterministic operations never invoke LLM binaries and reject invalid reports and incomplete eval arguments", () => {
  const { root, marker, invoke } = setup();
  expect(invoke(["validate-evals"]).exitCode).toBe(0);
  expect(invoke(["validate-evidence"]).exitCode).toBe(0);
  const smoke = invoke(["fixture-smoke"]);
  expect(smoke.exitCode).toBe(0);
  expect(reportSchema.parse(JSON.parse(smoke.stdout.toString())).agent).toBe(
    "fixture-smoke",
  );
  const malformed = join(root, "invalid.json");
  writeFileSync(malformed, "{}");
  expect(invoke(["validate-evidence", malformed]).exitCode).not.toBe(0);
  expect(invoke(["eval"]).exitCode).not.toBe(0);
  expect(
    invoke([
      "eval",
      "--model",
      "explicit",
      "--only",
      "code-search-literal",
      "--report",
      malformed,
    ]).exitCode,
  ).not.toBe(0);
  expect(readFileSync(malformed, "utf8")).toBe("{}");
  expect(existsSync(marker)).toBe(false);
});
