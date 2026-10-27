import type { Agent, HookKind } from "./install-agent-hooks.ts";
import { chmodSync, mkdirSync, writeFileSync } from "node:fs";
import {
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import type { DeploymentFixture } from "./deployment-test-support.ts";

const executableMode = 0o755;

function hooksFixture(
  agent: Agent,
  kinds: readonly HookKind[] = ["measurement"],
): Readonly<{ fixture: DeploymentFixture; destination: string }> {
  const fixture = createDeploymentFixture(`hook-config-${agent}`);
  mkdirSync(join(fixture.repository, "home"));
  writeHookManifest(fixture, agent, kinds);
  for (const executable of [
    "agent-measure",
    "agent-handoff",
    "output-discipline-hook",
  ]) {
    const path = join(fixture.home, ".local/bin", executable);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, "fixture executable");
    chmodSync(path, executableMode);
  }
  for (const executable of ["format-edited-file", "semctx-nudge"]) {
    const path = join(fixture.repository, "tooling", executable);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, "fixture executable");
    chmodSync(path, executableMode);
  }
  const destination = join(
    fixture.home,
    `.${agent}`,
    agent === "claude" ? "settings.json" : "hooks.json",
  );
  mkdirSync(dirname(destination), { recursive: true });
  return { fixture, destination };
}

function writeHookManifest(
  fixture: DeploymentFixture,
  agent: Agent,
  kinds: readonly HookKind[],
): void {
  writeFileSync(
    join(fixture.repository, "home/.arnes.yaml"),
    JSON.stringify({
      version: 1,
      hooks: kinds.map((id) => ({
        id,
        installations: [{ agent, scope: "user" }],
      })),
    }),
  );
}

function runHookInstaller(
  fixture: DeploymentFixture,
  agent: Agent,
): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(
    fixture,
    {
      helper: "install-agent-hooks.ts",
      arguments: [fixture.repository, fixture.home, agent],
    },
    { VOLTA_HOME: undefined, HOMEBREW_NO_AUTO_UPDATE: "1" },
  );
}

export { hooksFixture, runHookInstaller, writeHookManifest };
