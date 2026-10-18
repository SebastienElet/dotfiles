import { afterEach, expect, test } from "bun:test";
import { cleanup, run } from "./git-main-branch.test-support/harness.ts";

afterEach(cleanup);

const probeFailureExitCode = 42;
const unsupportedOptionExitCode = 2;

test.each([["main"], ["master"], ["trunk"]])(
  "selects local %s",
  async (branch) => {
    const result = await run({ localBranches: [branch] }, "-C", "/repository");
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toBe(`${branch}\n`);
  },
);

test("keeps permissive fallback outside a repository", async () => {
  const result = await run({ repository: false }, "-C", "/missing");
  expect([
    result.exitCode,
    result.stdout.toString(),
    result.stderr.toString(),
  ]).toEqual([0, "main\n", ""]);
});

test("strict mode exposes repository and branch-probe failures", async () => {
  const outside = await run(
    {
      failures: {
        "git rev-parse --git-dir": { status: 128, stderr: "fatal: absent\n" },
      },
      repository: false,
    },
    "--strict",
  );
  expect([
    outside.exitCode,
    outside.stdout.toString(),
    outside.stderr.toString(),
  ]).toEqual([1, "", "fatal: absent\n"]);
  const probe = await run({ showRefStatus: probeFailureExitCode }, "--strict");
  expect([probe.exitCode, probe.stdout.toString()]).toEqual([
    probeFailureExitCode,
    "",
  ]);
});

test("permissive fallback preserves branch-probe diagnostics", async () => {
  const result = await run({
    showRefStatus: probeFailureExitCode,
    showRefStderr: "fatal: probe\n",
  });
  expect([
    result.exitCode,
    result.stdout.toString(),
    result.stderr.toString(),
  ]).toEqual([0, "main\n", "fatal: probe\nfatal: probe\nfatal: probe\n"]);
});

test.each([
  ["alone", ["--bitbucket-cloud"]],
  ["after strict", ["--strict", "--bitbucket-cloud"]],
  ["after Git arguments", ["-C", "/repository", "--bitbucket-cloud"]],
  ["repeated", ["--bitbucket-cloud", "--bitbucket-cloud"]],
] as const)(
  "rejects the removed provider option: %s",
  async (_name: string, arguments_: readonly string[]) => {
    const result = await run({}, ...arguments_);
    expect([
      result.exitCode,
      result.stdout.toString(),
      result.stderr.toString(),
    ]).toEqual([
      unsupportedOptionExitCode,
      "",
      "git-main-branch: unsupported option --bitbucket-cloud\n",
    ]);
  },
);
