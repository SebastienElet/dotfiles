import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  installProvider,
  project,
  runMake,
} from "./deployment-test-support.ts";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { z } from "zod";

afterEach(cleanupDeploymentFixtures);

test("Codex aggregate target includes design claim audit", () => {
  const fixture = createDeploymentFixture("design-claim-audit");
  installProvider(fixture, "bun");
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

test("Codex migrates the managed design claim auditor symlink", () => {
  const fixture = createDeploymentFixture("design-claim-auditor-migration");
  const source = join(project, "home/.codex/agents/design-claim-auditor.toml");
  const destination = join(
    fixture.home,
    ".codex/agents/design-claim-auditor.toml",
  );
  mkdirSync(join(fixture.home, ".codex/agents"), { recursive: true });
  symlinkSync(source, destination);

  const first = runMake(fixture, [destination], { repository: project });
  const second = runMake(fixture, [destination], { repository: project });

  expectSuccess(first);
  expectSuccess(second);
  expect(lstatSync(destination).isSymbolicLink()).toBe(false);
  expect(readFileSync(destination, "utf8")).toBe(readFileSync(source, "utf8"));
  expect(second.stdout).toBe("");
  expect(second.stderr).toBe("");
});

test("Codex preserves an unrelated design claim auditor symlink", () => {
  const fixture = createDeploymentFixture("design-claim-auditor-wrong-link");
  const destination = join(
    fixture.home,
    ".codex/agents/design-claim-auditor.toml",
  );
  const unrelated = join(fixture.root, "unrelated.toml");
  mkdirSync(join(fixture.home, ".codex/agents"), { recursive: true });
  writeFileSync(unrelated, 'name = "unrelated"\n');
  symlinkSync(unrelated, destination);

  const result = runMake(fixture, [destination], { repository: project });

  expect(result.exitCode).not.toBe(0);
  expect(lstatSync(destination).isSymbolicLink()).toBe(true);
  expect(readFileSync(destination, "utf8")).toBe('name = "unrelated"\n');
});

test("Codex preserves a divergent regular design claim auditor file", () => {
  const fixture = createDeploymentFixture(
    "design-claim-auditor-divergent-file",
  );
  const destination = join(
    fixture.home,
    ".codex/agents/design-claim-auditor.toml",
  );
  mkdirSync(join(fixture.home, ".codex/agents"), { recursive: true });
  writeFileSync(destination, 'name = "local-override"\n');

  const result = runMake(fixture, [destination], { repository: project });

  expect(result.exitCode).not.toBe(0);
  expect(lstatSync(destination).isSymbolicLink()).toBe(false);
  expect(readFileSync(destination, "utf8")).toBe('name = "local-override"\n');
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
