import { afterEach, expect, test } from "bun:test";
import {
  cleanup,
  createExternalFile,
  createLinkedRegistry,
  createRegistry,
  readRegistry,
  repositoryRoot,
  runRegistryCli,
} from "./invariant-registry-cli.test-support.ts";
import { join, resolve } from "node:path";

const invalidUtf8Byte = 0xff;

afterEach(cleanup);

test("validates the canonical empty registry", async () => {
  const outcome = await runRegistryCli();

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toBe(
    "Invariant registry passed: harness/invariants/registry.json\n",
  );
  expect(outcome.stderr).toBe("");
});

test("locks the canonical registry to its empty structured value", async () => {
  expect(
    await readRegistry(
      join(repositoryRoot, "harness/invariants/registry.json"),
    ),
  ).toEqual({ invariants: [], version: 1 });
});

test.each([
  ["missing file", "missing.json", "unable to read invariant registry"],
  ["invalid JSON", "{", "valid JSON"],
  [
    "unknown version",
    '{"version":2,"invariants":[]}',
    "invalid invariant registry",
  ],
] as const)("fails closed for %s", async (_name, contents, diagnostic) => {
  const path =
    contents === "missing.json" ? contents : await createRegistry(contents);
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain(diagnostic);
});

test("fails closed for invalid UTF-8", async () => {
  const path = await createRegistry("\u0000");
  await Bun.write(
    resolve(repositoryRoot, path),
    Uint8Array.from([invalidUtf8Byte]),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("valid UTF-8");
});

test("refuses a registry symlink that resolves outside the repository", async () => {
  const target = await createExternalFile(
    "registry.json",
    '{"version":1,"invariants":[]}',
  );
  const outcome = await runRegistryCli(await createLinkedRegistry(target));

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("within the repository");
});

test("sanitizes schema failures from registry stderr", async () => {
  const secretField = "unrecognized-secret-field";
  const secretValue = "unrecognized-secret-value";
  const path = await createRegistry(
    JSON.stringify({
      [secretField]: secretValue,
      invariants: [],
      version: 1,
    }),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("invalid invariant registry");
  expect(outcome.stderr).not.toContain(secretField);
  expect(outcome.stderr).not.toContain(secretValue);
});

test.each([
  "../outside.json",
  "/private/tmp/outside.json",
  String.raw`C:\outside.json`,
] as const)("refuses an input path outside the repository", async (path) => {
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("within the repository");
});
