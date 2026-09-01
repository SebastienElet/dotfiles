import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  installProvider,
  project,
  runMake,
} from "./deployment-test-support.ts";
import { lstatSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

afterEach(cleanupDeploymentFixtures);

test("Codex aggregate target includes design claim audit", () => {
  const fixture = createDeploymentFixture("design-claim-audit");
  installProvider(fixture, "bun");
  installProvider(fixture, "cargo");
  installProvider(fixture, "volta");
  const result = runMake(fixture, ["codex"], {
    dryRun: true,
    repository: project,
    variables: { BREW_BIN: fixture.bin },
  });
  expectSuccess(result);
  expect(result.stdout).toContain(
    join(fixture.home, ".agents/skills/design-claim-audit"),
  );
  expect(result.stdout).toContain(
    join(fixture.home, ".codex/agents/design-claim-auditor.toml"),
  );
});

test("Codex deploys the design claim auditor as a regular file", () => {
  const fixture = createDeploymentFixture("design-claim-auditor-regular-file");
  const destination = join(
    fixture.home,
    ".codex/agents/design-claim-auditor.toml",
  );
  const result = runMake(fixture, [destination], { repository: project });

  expectSuccess(result);
  expect(lstatSync(destination).isSymbolicLink()).toBe(false);
  expect(readFileSync(destination, "utf8")).toBe(
    readFileSync(
      join(project, "home/.codex/agents/design-claim-auditor.toml"),
      "utf8",
    ),
  );
});

test("design claim auditor declares its canonical role and execution defaults", () => {
  const config = z
    .object({
      name: z.string(),
      model: z.string(),
      sandbox_mode: z.string(),
      developer_instructions: z.string(),
    })
    .parse(
      Bun.TOML.parse(
        readFileSync(
          join(project, "home/.codex/agents/design-claim-auditor.toml"),
          "utf8",
        ),
      ),
    );
  const skill = readFileSync(
    join(project, "harness/skills/design-claim-audit/SKILL.md"),
    "utf8",
  );

  expect(config.name).toBe("design-claim-auditor");
  expect(config.model).toBe("gpt-5.6-sol");
  expect(config.sandbox_mode).toBe("read-only");
  expect(config.developer_instructions).toContain(
    "Never activate design-claim-audit",
  );
  expect(config.developer_instructions).toContain("Never spawn another agent");
  expect(skill).toContain("`design-claim-auditor`");
});
