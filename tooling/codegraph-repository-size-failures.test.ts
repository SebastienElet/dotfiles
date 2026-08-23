import { afterEach, describe, expect, test } from "bun:test";
import { chmodSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  cleanupFixtures,
  createFixture,
  readArguments,
  runEntryPoint,
} from "./codegraph-repository-size-test-support.ts";

afterEach(cleanupFixtures);

describe("codegraph-repository-size failures", () => {
  for (const operation of ["rev-parse", "ls-files"]) {
    test(`fails closed without publishing a measurement when Git ${operation} fails`, () => {
      const fixture = createFixture();
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_GIT_BIN: fixture.fakeGit,
        CODEGRAPH_TEST_GIT_REPOSITORY: "1",
        CODEGRAPH_TEST_GIT_FAILURE: operation,
      });

      expect(result.exitCode).toBe(7);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(`${operation} operational failure`);
      expect(readArguments(fixture.argumentsLog)).toHaveLength(
        operation === "rev-parse" ? 1 : 2,
      );
    });
  }

  for (const invalidPath of [
    {
      label: "truncated",
      environment: { CODEGRAPH_TEST_GIT_FILES: "truncated.tofu" },
      diagnostic: "NUL-terminated",
    },
    {
      label: "escaping",
      environment: {
        CODEGRAPH_TEST_GIT_FILES_JSON: '["../escaping.tofu"]',
      },
      diagnostic: "outside repository",
    },
  ] satisfies Array<{
    label: string;
    environment: Record<string, string>;
    diagnostic: string;
  }>) {
    test(`rejects ${invalidPath.label} Git paths before Tokei runs`, () => {
      const fixture = createFixture();
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_GIT_BIN: fixture.fakeGit,
        CODEGRAPH_TEST_GIT_REPOSITORY: "1",
        ...invalidPath.environment,
      });

      expect(result.exitCode).toBe(2);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(invalidPath.diagnostic);
      expect(readArguments(fixture.argumentsLog)).toHaveLength(2);
    });
  }

  test("fails closed when Tokei emits partial output and fails", () => {
    const fixture = createFixture();
    const result = runEntryPoint(fixture.repository, {
      ...fixture.environment,
      CODEGRAPH_TEST_TOKEI_FAILURE: "7",
    });

    expect(result.exitCode).toBe(7);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("tokei operational failure");
  });

  test("rejects missing dependencies before measurement", () => {
    const fixture = createFixture();
    for (const [name, environment] of [
      [
        "Tokei",
        { CODEGRAPH_TOKEI_BIN: join(fixture.directory, "missing-tokei") },
      ],
      ["Git", { CODEGRAPH_GIT_BIN: join(fixture.directory, "missing-git") }],
    ] as const) {
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        ...environment,
      });
      expect(result.exitCode).toBe(2);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(`${name} is required`);
    }
  });

  test("rejects missing, non-directory, and unreadable repositories", () => {
    const fixture = createFixture();
    const file = join(fixture.directory, "file");
    const unreadable = join(fixture.directory, "unreadable");
    writeFileSync(file, "file\n");
    mkdirSync(unreadable);
    chmodSync(unreadable, 0o000);
    try {
      for (const path of [
        join(fixture.directory, "missing"),
        file,
        unreadable,
      ]) {
        const result = runEntryPoint(path, fixture.environment);
        expect(result.exitCode).toBe(2);
        expect(result.stdout).toBe("");
        expect(result.stderr).toContain("repository");
      }
    } finally {
      chmodSync(unreadable, 0o700);
    }
  });

  test("rejects invalid Tokei output without publishing a plausible default", () => {
    const fixture = createFixture();
    for (const output of ["not-json\n", '{"stats":{}}\n']) {
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_TEST_TOKEI_OUTPUT: output,
      });
      expect(result.exitCode).toBe(2);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain("invalid Tokei");
    }
  });
});
