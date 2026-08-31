import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  project,
  runMake,
} from "./deployment-test-support.ts";
import { join } from "node:path";
import { readFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);

const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

test("Bun consumers reuse the shared Homebrew target exactly once", () => {
  const fixture = createDeploymentFixture("bun-wiring");
  const brewBin = join(fixture.home, "homebrew", "bin");
  for (const target of ["claude-code", "codex", "obsidian-retrieval-test"]) {
    const result = runMake(fixture, [target], {
      dryRun: true,
      repository: project,
      variables: { BREW_BIN: brewBin },
    });
    expectSuccess(result);
    expect(count(result.stdout, "brew install bun")).toBe(1);
  }
});

test.each([
  ["codex", "codex", true],
  ["claude-code", "claude", true],
  ["cursor", "cursor", false],
] as const)(
  "%s entry point reconciles hooks once",
  (target, agent, deploysHandoff) => {
    const fixture = createDeploymentFixture(`hook-wiring-${target}`);
    const expected = `"${fixture.home}/.local/bin/arnes" setup hooks --agent ${agent}`;
    const hook = runMake(fixture, [`${target}-hooks`], {
      dryRun: true,
      repository: project,
    });
    expectSuccess(hook);
    expect(count(hook.stdout, expected)).toBe(1);
    expect(hook.stdout).toContain("cargo build --release");
    const memory = join(fixture.home, ".local", "bin", "agent-memory");
    const handoff = join(fixture.home, ".local", "bin", "agent-handoff");
    expect(hook.stdout).toContain(memory);
    expectRuntime(hook.stdout, handoff, deploysHandoff);
    const entry = runMake(fixture, [target], {
      dryRun: true,
      repository: project,
    });
    expectSuccess(entry);
    expect(count(entry.stdout, expected)).toBe(1);
    expect(entry.stdout).toContain(memory);
    expectRuntime(entry.stdout, handoff, deploysHandoff);
    if (target === "codex") {
      expect(entry.stdout).toContain(
        join(project, "harness", "skills", "agent-instructions"),
      );
    }
  },
);

test("declares memory hooks for Codex and Claude only", () => {
  const manifest = readFileSync(join(project, "home", ".arnes.yaml"), "utf8");

  expect(manifest).toMatch(
    /- id: memory\n\s+installations:\n\s+- \{ agent: claude, scope: user \}\n\s+- \{ agent: codex, scope: user \}/u,
  );
});

test("deploys the Cursor memory rule from its canonical source", () => {
  const fixture = createDeploymentFixture("cursor-memory-rule");
  const source = join(project, "harness/rules/memory-governance-cursor.mdc");
  const destination = join(
    fixture.home,
    ".cursor/rules/memory-governance-cursor.mdc",
  );

  expectSuccess(runMake(fixture, [destination], { repository: project }));
  expect(linkTarget(destination)).toBe(source);
  const rule = readFileSync(source, "utf8");
  expect(rule).toContain("alwaysApply: true");
  expect(rule).toContain("Load the `memory-governance` skill");
  expect(rule).toContain("agent-memory retrieve --query-stdin --format json");
  expect(rule).toContain("wait for completion");
  expect(rule).toContain("apply no memory");
  expect(rule).not.toMatch(/schema_version|ranking|privacy policy/u);
});

test("deploys independent memory and handoff runtime binaries", () => {
  const fixture = createDeploymentFixture("memory-runtime-binaries");
  const memory = join(fixture.home, ".local/bin/agent-memory");
  const handoff = join(fixture.home, ".local/bin/agent-handoff");
  const result = (target: string): ReturnType<typeof runMake> =>
    runMake(fixture, [target], { dryRun: true, repository: project });

  const memoryResult = result("agent-memory");
  const handoffResult = result("agent-handoff");
  expectSuccess(memoryResult);
  expectSuccess(handoffResult);
  expect(memoryResult.stdout).toContain(memory);
  expect(memoryResult.stdout).not.toContain(handoff);
  expect(handoffResult.stdout).toContain(handoff);
  expect(handoffResult.stdout).not.toContain(memory);
});

test("installs the formula, runtime, four pinned dictionaries, and Claude dependency", () => {
  const fixture = createDeploymentFixture("hunspell-wiring");
  const revision = "f2ff99058268502bdcf4cad25c1ca2935ad8aa7d";
  const base = `https://raw.githubusercontent.com/LibreOffice/dictionaries/${revision}`;
  const result = runMake(fixture, ["hunspell"], {
    dryRun: true,
    repository: project,
    variables: { BREW_BIN: fixture.bin },
  });
  expectSuccess(result);
  expect(count(result.stdout, "brew install hunspell")).toBe(1);
  expect(count(result.stdout, "brew install bun")).toBe(1);
  for (const [source, checksum, destination] of [
    [
      "fr_FR/dictionaries/fr.aff",
      "c176610cd5dc4846806a65ddd029f422d87978bf58f224aa44222662a16a2de5",
      "fr.aff",
    ],
    [
      "fr_FR/dictionaries/fr.dic",
      "b78a868e31dd6e373b6c3217969afb898a9acde828a5e7ef97308da42218c88c",
      "fr.dic",
    ],
    [
      "en/en_US.aff",
      "e746c882dd6f303c2c46e7452804b9201115a6942cfeb15f18f8edf774d2e24e",
      "en_US.aff",
    ],
    [
      "en/en_US.dic",
      "f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647",
      "en_US.dic",
    ],
  ] as const) {
    const expected = `"${join(project, "tooling", "install-hunspell-dictionary")}" "${base}/${source}" "${checksum}" "${join(fixture.home, "Library", "Spelling", destination)}"`;
    expect(count(result.stdout, expected)).toBe(1);
  }
  const claude = runMake(fixture, ["claude-code"], {
    dryRun: true,
    repository: project,
    variables: { BREW_BIN: fixture.bin },
  });
  expectSuccess(claude);
  expect(claude.stdout).toContain("brew install hunspell");
});

function count(value: string, needle: string): number {
  return value.split(needle).length - 1;
}

function expectRuntime(output: string, path: string, present: boolean): void {
  if (present) {
    expect(output).toContain(path);
    return;
  }
  expect(output).not.toContain(path);
}
