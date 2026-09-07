import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  project,
} from "./deployment-test-support.ts";
import { lstatSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);

test("Codex deploys the design claim auditor as a regular file", () => {
  const fixture = createDeploymentFixture("design-claim-auditor-regular-file");
  const destination = join(
    fixture.home,
    ".codex/agents/design-claim-auditor.toml",
  );
  const result = runDeploymentMoon(fixture, ["harness:codex-agents"]);

  expectSuccess(result);
  expect(lstatSync(destination).isSymbolicLink()).toBe(false);
  expect(readFileSync(destination, "utf8")).toBe(
    readFileSync(
      join(project, "home/.codex/agents/design-claim-auditor.toml"),
      "utf8",
    ),
  );
});
