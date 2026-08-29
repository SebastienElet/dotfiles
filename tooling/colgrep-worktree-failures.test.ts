import {
  createLinkedWorktreeFixture,
  gitProvider,
  readInvocations,
  runEntryPoint,
} from "./colgrep-worktree-test-support.ts";
import { expect, test } from "bun:test";

test("refuses a main checkout before ColGrep", () => {
  const fixture = createLinkedWorktreeFixture();

  expectRefusal(runEntryPoint(fixture.mainRoot, "query", fixture.environment));
  expect(readInvocations(fixture)).toEqual([]);
});

test("refuses a non-Git directory before ColGrep", () => {
  const fixture = createLinkedWorktreeFixture();

  expectRefusal(runEntryPoint(fixture.root, "query", fixture.environment));
  expect(readInvocations(fixture)).toEqual([]);
});

test("requires exactly one non-empty conceptual query", () => {
  const fixture = createLinkedWorktreeFixture();

  expectRefusal(runEntryPoint(fixture.linkedRoot, "", fixture.environment));
  expect(readInvocations(fixture)).toEqual([]);
});

test("fails closed when Git is absent", () => {
  const fixture = createLinkedWorktreeFixture();
  const environment = {
    ...fixture.environment,
    COLGREP_WORKTREE_GIT_BIN: `${fixture.root}/missing-git`,
  };

  expectRefusal(runEntryPoint(fixture.linkedRoot, "query", environment));
  expect(readInvocations(fixture)).toEqual([]);
});

test("fails closed when ColGrep is absent", () => {
  const fixture = createLinkedWorktreeFixture();
  const environment = {
    ...fixture.environment,
    COLGREP_WORKTREE_COLGREP_BIN: `${fixture.root}/missing-colgrep`,
  };

  expectRefusal(runEntryPoint(fixture.linkedRoot, "query", environment));
  expect(readInvocations(fixture)).toEqual([]);
});

test("ignores inherited Git routing variables", () => {
  const fixture = createLinkedWorktreeFixture();
  const environment = {
    ...fixture.environment,
    GIT_COMMON_DIR: `${fixture.root}/foreign-common`,
    GIT_DIR: `${fixture.mainRoot}/.git`,
    GIT_INDEX_FILE: `${fixture.root}/foreign-index`,
    GIT_PREFIX: "foreign",
    GIT_WORK_TREE: fixture.mainRoot,
  };

  const result = runEntryPoint(fixture.linkedRoot, "query", environment);

  expect(result.exitCode).toBe(0);
  expect(JSON.parse(result.stdout)).toEqual([fixture.activeResult]);
});

test.each([
  ["empty-root", "Git worktree root is missing or ambiguous"],
  ["multiple-root", "Git worktree root is missing or ambiguous"],
  ["superproject", "the active repository is not a linked worktree"],
])("%s Git evidence refuses before ColGrep", (mode, expectedError) => {
  const fixture = createLinkedWorktreeFixture();
  const environment = {
    ...fixture.environment,
    COLGREP_TEST_GIT_MODE: mode,
    COLGREP_WORKTREE_GIT_BIN: gitProvider,
  };

  const result = runEntryPoint(fixture.linkedRoot, "query", environment);
  expectRefusal(result);
  expect(result.stderr).toContain(expectedError);
  expect(readInvocations(fixture)).toEqual([]);
});

test.each([
  "init-failure",
  "missing-index",
  "ambiguous-status",
  "foreign-status",
  "symlink-index",
  "malformed-project",
  "foreign-project",
  "dirty-state",
  "empty-state",
  "malformed-state",
])("%s refuses before index search", (mode) => {
  const fixture = createLinkedWorktreeFixture({ mode });

  expectRefusal(
    runEntryPoint(fixture.linkedRoot, "query", fixture.environment),
  );
  expect(
    readInvocations(fixture).some((invocation: readonly string[]) =>
      invocation.includes("search"),
    ),
  ).toBeFalse();
});

test.each([
  "search-failure",
  "malformed-results",
  "relative-result",
  "foreign-result",
])("%s publishes no partial result", (mode) => {
  const fixture = createLinkedWorktreeFixture({ mode });

  expectRefusal(
    runEntryPoint(fixture.linkedRoot, "query", fixture.environment),
  );
  expect(
    readInvocations(fixture).some((invocation: readonly string[]) =>
      invocation.includes("search"),
    ),
  ).toBeTrue();
});

function expectRefusal(result: {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
}): void {
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).toBe("");
  expect(result.stderr).toEndWith("Fall back to bounded rg/fd searches.\n");
}
