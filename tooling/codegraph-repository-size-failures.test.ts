import { afterEach, describe, expect, test } from "bun:test";
import { chmodSync, mkdirSync, writeFileSync } from "node:fs";
import {
  cleanupFixtures,
  createFixture,
  readArguments,
  runEntryPoint,
} from "./codegraph-repository-size-test-support.ts";
import { join } from "node:path";

const measurementErrorExitCode = 2;
const operationalFailureExitCode = 7;
const expectedGitCommandCount = 2;
const restoredDirectoryMode = 0o700;

afterEach(cleanupFixtures);

describe("codegraph-repository-size failures", () => {
  registerGitFailureTests();
  registerInvalidPathTests();
  registerInvalidOutputTests();
  registerDependencyTests();
  registerRepositoryTests();
});

function registerGitFailureTests(): void {
  for (const operation of ["rev-parse", "ls-files"]) {
    test(`fails closed without publishing a measurement when Git ${operation} fails`, () => {
      const fixture = createFixture();
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_GIT_BIN: fixture.fakeGit,
        CODEGRAPH_TEST_GIT_FAILURE: operation,
        CODEGRAPH_TEST_GIT_REPOSITORY: "1",
      });

      expect(result.exitCode).toBe(operationalFailureExitCode);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(`${operation} operational failure`);
      expect(readArguments(fixture.argumentsLog)).toHaveLength(
        operation === "rev-parse" ? 1 : expectedGitCommandCount,
      );
    });
  }
}

function registerInvalidPathTests(): void {
  for (const invalidPath of [
    {
      diagnostic: "NUL-terminated",
      environment: { CODEGRAPH_TEST_GIT_FILES: "truncated.tofu" },
      label: "truncated",
    },
    {
      diagnostic: "outside repository",
      environment: {
        CODEGRAPH_TEST_GIT_FILES_JSON: '["../escaping.tofu"]',
      },
      label: "escaping",
    },
  ] satisfies {
    label: string;
    environment: Record<string, string>;
    diagnostic: string;
  }[]) {
    test(`rejects ${invalidPath.label} Git paths before Tokei runs`, () => {
      const fixture = createFixture();
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_GIT_BIN: fixture.fakeGit,
        CODEGRAPH_TEST_GIT_REPOSITORY: "1",
        ...invalidPath.environment,
      });

      expect(result.exitCode).toBe(measurementErrorExitCode);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(invalidPath.diagnostic);
      expect(readArguments(fixture.argumentsLog)).toHaveLength(
        expectedGitCommandCount,
      );
    });
  }
}

function registerInvalidOutputTests(): void {
  test("fails closed when Tokei emits partial output and fails", () => {
    const fixture = createFixture();
    const result = runEntryPoint(fixture.repository, {
      ...fixture.environment,
      CODEGRAPH_TEST_TOKEI_FAILURE: "7",
    });

    expect(result.exitCode).toBe(operationalFailureExitCode);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("tokei operational failure");
  });

  test("rejects invalid UTF-8 from Git without publishing a measurement", () => {
    const fixture = createFixture();
    const result = runEntryPoint(fixture.repository, {
      ...fixture.environment,
      CODEGRAPH_GIT_BIN: fixture.fakeGit,
      CODEGRAPH_TEST_GIT_INVALID_UTF8: "1",
      CODEGRAPH_TEST_GIT_REPOSITORY: "1",
    });

    expect(result.exitCode).toBe(measurementErrorExitCode);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("invalid UTF-8");
    expect(readArguments(fixture.argumentsLog)).toHaveLength(
      expectedGitCommandCount,
    );
  });
}

function registerDependencyTests(): void {
  test("rejects invalid UTF-8 from Tokei without publishing a measurement", () => {
    const fixture = createFixture();
    const result = runEntryPoint(fixture.repository, {
      ...fixture.environment,
      CODEGRAPH_TEST_TOKEI_INVALID_UTF8: "1",
    });

    expect(result.exitCode).toBe(measurementErrorExitCode);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("invalid UTF-8");
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
      expect(result.exitCode).toBe(measurementErrorExitCode);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(`${name} is required`);
    }
  });
}

function registerRepositoryTests(): void {
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
        expect(result.exitCode).toBe(measurementErrorExitCode);
        expect(result.stdout).toBe("");
        expect(result.stderr).toContain("repository");
      }
    } finally {
      chmodSync(unreadable, restoredDirectoryMode);
    }
  });

  test("rejects invalid Tokei output without publishing a plausible default", () => {
    const fixture = createFixture();
    for (const output of ["not-json\n", '{"stats":{}}\n']) {
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_TEST_TOKEI_OUTPUT: output,
      });
      expect(result.exitCode).toBe(measurementErrorExitCode);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain("invalid Tokei");
    }
  });
}
