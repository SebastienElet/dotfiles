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

test.each([
  ["codex", "codex", true],
  ["claude-code", "claude", true],
  ["cursor", "cursor", false],
] as const)(
  "%s entry point deploys its memory runtime",
  (target, agent, deploysHandoff) => {
    const fixture = createDeploymentFixture(`memory-wiring-${target}`);
    const memory = join(fixture.home, ".local", "bin", "agent-memory");
    const handoff = join(fixture.home, ".local", "bin", "agent-handoff");
    const expected = `"${fixture.home}/.local/bin/arnes" setup hooks --agent ${agent}`;

    const result = runMake(fixture, [target], {
      dryRun: true,
      repository: project,
    });

    expectSuccess(result);
    expect(result.stdout).toContain(memory);
    expect(result.stdout.includes(handoff)).toBe(deploysHandoff);
    expect(result.stdout).toContain(expected);
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
  expect(rule).toContain("agent-memory retrieve --query-stdin --format json");
  expect(rule).toContain("wait for completion");
  expect(rule).toContain("apply no memory");
  expect(rule).not.toMatch(/schema_version|ranking|privacy policy/u);
});

test("keeps memory and handoff runtime targets independent", () => {
  const fixture = createDeploymentFixture("memory-runtime-binaries");
  const memory = join(fixture.home, ".local", "bin", "agent-memory");
  const handoff = join(fixture.home, ".local", "bin", "agent-handoff");
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
