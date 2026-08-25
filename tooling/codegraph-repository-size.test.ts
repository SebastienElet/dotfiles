import { afterEach, describe, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  readArguments,
  run,
  runEntryPoint,
} from "./codegraph-repository-size-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { z } from "zod";

const measurementSchema = z.object({ files: z.number() });
const fixtureSourceLines = 12;

afterEach(cleanupFixtures);

describe("codegraph-repository-size entry point", () => {
  registerMeasurementTests();
  registerGitSelectionTest();
  registerFilesystemSelectionTest();
  registerNonGitSelectionTest();
  registerRealTokeiTest();
  registerDeploymentTest();
});

function registerMeasurementTests(): void {
  test("reports an empty non-Git repository without initialization", () => {
    const fixture = createFixture();

    expect(runEntryPoint(fixture.repository, fixture.environment)).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: '{"loc":0,"files":0,"initialize":false}\n',
    });
  });
}

function registerGitSelectionTest(): void {
  test("publishes the stable schema and forwards the preserved Tokei policy", () => {
    const fixture = createFixture();
    const result = runEntryPoint(fixture.repository, {
      ...fixture.environment,
      CODEGRAPH_TEST_TOKEI_OUTPUT: record("src/fixture.ts", fixtureSourceLines),
    });

    expect(result).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: `{"loc":${fixtureSourceLines},"files":1,"initialize":false}\n`,
    });
    const [arguments_] = readArguments(fixture.argumentsLog);
    expect(arguments_).toContain("--hidden");
    expect(arguments_).toContain("--streaming");
    expect(arguments_).toContain("node_modules");
    expect(arguments_).toContain("docs");
    expect(arguments_).toContain("fixtures");
    expect(arguments_?.join(" ")).toContain("TypeScript");
    expect(arguments_?.join(" ")).toContain("Razor");
  });
}

function registerFilesystemSelectionTest(): void {
  test("selects only eligible OpenTofu files in a Git repository", () => {
    const fixture = createFixture();
    mkdirSync(join(fixture.repository, "src"));
    mkdirSync(join(fixture.repository, "docs"));
    mkdirSync(join(fixture.repository, "ignored"));
    writeFileSync(join(fixture.repository, ".gitignore"), "ignored/\n");
    writeFileSync(join(fixture.repository, "src", "kept.tofu"), "kept\n");
    writeFileSync(join(fixture.repository, "docs", "excluded.tofu"), "docs\n");
    writeFileSync(
      join(fixture.repository, "ignored", "ignored.tofu"),
      "ignored\n",
    );
    symlinkSync(
      join(fixture.repository, "src", "kept.tofu"),
      join(fixture.repository, "linked.tofu"),
    );
    expect(run(["git", "init", "-q"], fixture.repository).exitCode).toBe(0);
    expect(
      run(
        [
          "git",
          "add",
          ".gitignore",
          "src/kept.tofu",
          "docs/excluded.tofu",
          "linked.tofu",
        ],
        fixture.repository,
      ).exitCode,
    ).toBe(0);

    const result = runEntryPoint(fixture.repository, fixture.environment);

    expect(result.exitCode).toBe(0);
    expect(JSON.parse(result.stdout)).toEqual({
      files: 1,
      initialize: false,
      loc: 1,
    });
    const tofuCall = readArguments(fixture.argumentsLog).find(
      (arguments_: readonly string[]) =>
        arguments_.some((argument) => argument.endsWith(".tf")),
    );
    expect(
      tofuCall?.filter((argument) => argument.endsWith(".tf")),
    ).toHaveLength(1);
  });
}

function registerNonGitSelectionTest(): void {
  test("selects OpenTofu files without Git and ignores symlinks and excluded trees", () => {
    const fixture = createFixture();
    mkdirSync(join(fixture.repository, "src"));
    mkdirSync(join(fixture.repository, "vendor"));
    writeFileSync(join(fixture.repository, "src", "kept.ToFu"), "kept\n");
    writeFileSync(
      join(fixture.repository, "vendor", "excluded.tofu"),
      "excluded\n",
    );
    symlinkSync(
      join(fixture.repository, "src", "kept.ToFu"),
      join(fixture.repository, "linked.tofu"),
    );
    const result = runEntryPoint(fixture.repository, fixture.environment);

    expect(result.exitCode).toBe(0);
    expect(measurementSchema.parse(JSON.parse(result.stdout)).files).toBe(1);
  });
}

function registerRealTokeiTest(): void {
  test.skipIf(Bun.which("tokei") === null)(
    "matches real Tokei exclusions and supported extensions",
    () => {
      const fixture = createFixture();
      populateRealTokeiFixture(fixture.repository);
      expect(run(["git", "init", "-q"], fixture.repository).exitCode).toBe(0);

      const tokeiBinary = Bun.which("tokei");
      if (tokeiBinary === null) {
        throw new Error("tokei is required");
      }
      const result = runEntryPoint(fixture.repository, {
        ...fixture.environment,
        CODEGRAPH_TOKEI_BIN: tokeiBinary,
      });

      expect(result.exitCode).toBe(0);
      expect(JSON.parse(result.stdout)).toEqual({
        files: 3,
        initialize: false,
        loc: 3,
      });
    },
  );
}

function populateRealTokeiFixture(repository: string): void {
  mkdirSync(join(repository, "src"));
  mkdirSync(join(repository, "docs"));
  mkdirSync(join(repository, "ignored"));
  writeFileSync(join(repository, ".gitignore"), "ignored/\n");
  writeFileSync(join(repository, "src", "kept.ts"), "const kept = 1;\n");
  writeFileSync(join(repository, "src", "kept.cjs"), "exports.kept = 1;\n");
  writeFileSync(
    join(repository, "src", "kept.tofu"),
    'resource "fixture" "kept" {}\n',
  );
  writeFileSync(join(repository, "src", "excluded.hcl"), "kept = true\n");
  writeFileSync(join(repository, "docs", "excluded.ts"), "const docs = 1;\n");
  writeFileSync(
    join(repository, "ignored", "excluded.ts"),
    "const ignored = 1;\n",
  );
}

function registerDeploymentTest(): void {
  test("the Make deployment preserves the command name and provisions dependencies", () => {
    const fixture = createFixture();
    const root = dirname(import.meta.dir);
    const result = run([
      "make",
      "-sBn",
      "-C",
      root,
      "codegraph",
      `HOME=${fixture.directory}`,
      `LOCAL_BIN=${join(fixture.directory, ".local", "bin")}`,
      `BREW_BIN=${join(fixture.directory, "brew", "bin")}`,
      `VOLTA_BIN=${join(fixture.directory, ".volta", "bin")}`,
    ]);

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toContain("brew install tokei");
    expect(result.stdout).toContain("tooling/codegraph-repository-size");
    expect(result.stdout).toContain("codegraph-repository-size");
  });
}

function record(name: string, code: number): string {
  return `${JSON.stringify({ stats: { name, stats: { code } } })}\n`;
}
