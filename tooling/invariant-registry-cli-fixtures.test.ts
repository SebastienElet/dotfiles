import { afterEach, expect, test } from "bun:test";
import {
  cleanup,
  createExternalFile,
  createLinkedOracle,
  createUntrackedOracle,
  fixturePath,
  mutatedFixture,
  readRegistry,
  runRegistryCli,
} from "./invariant-registry-cli.test-support.ts";

const pr206Fixture = "pr-206-secret-redaction.json";
const pr207Fixture = "pr-207-invalid-utf8.json";

afterEach(cleanup);

test("validates the active PR 206 fixture", async () => {
  const outcome = await runRegistryCli(
    "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
  );

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
  expect(outcome.stderr).toBe("");
});

test("locks the PR 206 fixture to its historical structured value", async () => {
  expect(await readRegistry(fixturePath(pr206Fixture))).toEqual({
    invariants: [
      {
        approval: {
          approvedAt: "2026-09-02T00:00:00.000Z",
          approvedBy: "Sebastien",
        },
        causeClass: "not-applied",
        consumers: {
          claude: { reason: "Historical fixture only.", state: "unsupported" },
          codex: { reason: "Historical fixture only.", state: "unsupported" },
          cursor: { reason: "Historical fixture only.", state: "unsupported" },
        },
        controlKind: "enforceable",
        id: "prevent-fetch-url-secret-redaction",
        lifecycle: "active",
        oracle: {
          failurePath: "rejected fetch URL userinfo never reaches stderr",
          invocation: ["bun", "test", "tooling/git-main-branch-entry.test.ts"],
          name: "fetch-url-userinfo-redaction",
          testPath: "tooling/git-main-branch-entry.test.ts",
        },
        scope: { exceptions: [], kind: "project-local" },
        severity: "high",
        sources: [
          {
            evidenceUrl:
              "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552",
            provider: "github",
            pullRequestUrl:
              "https://github.com/SebastienElet/dotfiles/pull/206",
          },
        ],
        statement: "Rejected fetch URL userinfo never reaches stderr.",
        surface: "architectural-test",
        verification: { state: "unverified" },
      },
    ],
    version: 1,
  });
});

test("rejects the PR 206 fixture when its active oracle is absent", async () => {
  const path = await mutatedFixture(pr206Fixture, (source) =>
    source.replace(
      "tooling/git-main-branch-entry.test.ts",
      "tooling/missing-oracle.test.ts",
    ),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("Oracle test path does not exist");
});

test("rejects the PR 206 fixture when its active oracle resolves outside", async () => {
  const target = await createExternalFile("oracle.test.ts", "");
  const oraclePath = await createLinkedOracle(target);
  const path = await mutatedFixture(pr206Fixture, (source) =>
    source.replace("tooling/git-main-branch-entry.test.ts", oraclePath),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("regular file");
});

test("rejects an oracle symlink to an internal tracked test", async () => {
  const oraclePath = await createLinkedOracle(
    fixturePath("../git-main-branch-entry.test.ts"),
  );
  const path = await mutatedFixture(pr206Fixture, (source) =>
    source.replaceAll("tooling/git-main-branch-entry.test.ts", oraclePath),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).toContain("regular file");
});

test.each([
  ["directory", "tooling", "regular file"],
  ["non-test file", "package.json", "discovered by the test suite"],
] as const)(
  "rejects a PR 206 oracle backed by a %s",
  async (_name, oraclePath, diagnostic) => {
    const path = await mutatedFixture(pr206Fixture, (source) =>
      source.replaceAll("tooling/git-main-branch-entry.test.ts", oraclePath),
    );
    const outcome = await runRegistryCli(path);

    expect(outcome.exitCode).not.toBe(0);
    expect(outcome.stderr).toContain(diagnostic);
  },
);

test("rejects a PR 206 oracle that is not tracked by Git", async () => {
  const oraclePath = await createUntrackedOracle();
  const path = await mutatedFixture(pr206Fixture, (source) =>
    source.replaceAll("tooling/git-main-branch-entry.test.ts", oraclePath),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).toContain("tracked by Git");
});

test("validates the retired PR 207 fixture", async () => {
  const outcome = await runRegistryCli(
    "tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json",
  );

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
  expect(outcome.stderr).toBe("");
});

test("locks the PR 207 fixture to its historical structured value", async () => {
  expect(await readRegistry(fixturePath(pr207Fixture))).toEqual({
    invariants: [
      {
        causeClass: "blind-spot",
        consumers: {
          claude: { reason: "Historical fixture only.", state: "unsupported" },
          codex: { reason: "Historical fixture only.", state: "unsupported" },
          cursor: { reason: "Historical fixture only.", state: "unsupported" },
        },
        controlKind: "enforceable",
        id: "reject-invalid-utf8-measurement-output",
        lifecycle: "retired",
        retirement: {
          reason: "The historical repository measurement consumer was retired.",
          retiredAt: "2026-09-02T00:00:00.000Z",
        },
        scope: { exceptions: [], kind: "project-local" },
        severity: "high",
        sources: [
          {
            evidenceUrl:
              "https://github.com/SebastienElet/dotfiles/pull/207#issuecomment-5388145825",
            provider: "github",
            pullRequestUrl:
              "https://github.com/SebastienElet/dotfiles/pull/207",
          },
        ],
        statement:
          "The historical repository measurement consumer rejects invalid UTF-8 output.",
        surface: "architectural-test",
        verification: {
          lastRun: {
            environment: "macOS",
            outcome: "passed",
            ranAt: "2026-09-02T00:00:00.000Z",
          },
          state: "measured",
        },
      },
    ],
    version: 1,
  });
});

test("rejects the PR 207 fixture without a retirement reason", async () => {
  const path = await mutatedFixture(pr207Fixture, (source) =>
    source.replace(
      "The historical repository measurement consumer was retired.",
      " ",
    ),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain("invalid invariant registry");
});
