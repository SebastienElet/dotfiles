import type {
  CommandResult,
  DeploymentFixture,
} from "./deployment-test-support.ts";
import { chmodSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { runDeploymentHelper } from "./deployment-test-support.ts";

const executableMode = 0o755;

type LanternFixture = DeploymentFixture &
  Readonly<{
    plugin: string;
    registration: string;
    destination: string;
    generated: string;
  }>;

function createPlugin(
  root: string,
): Readonly<{ plugin: string; revision: string }> {
  const plugin = join(root, "plugin");
  mkdirSync(plugin);
  writeFileSync(
    join(plugin, "prompt.md"),
    "# Upstream\nKeep every upstream instruction.\n",
  );
  for (const args of [
    ["init", "--quiet", plugin],
    ["-C", plugin, "add", "prompt.md"],
    [
      "-C",
      plugin,
      "-c",
      "core.hooksPath=/dev/null",
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.test",
      "commit",
      "--quiet",
      "-m",
      "fixture",
    ],
  ]) {
    if (Bun.spawnSync(["git", ...args]).exitCode !== 0) {
      throw new Error("Cannot initialize fixture plugin");
    }
  }
  const revision = Bun.spawnSync(["git", "-C", plugin, "rev-parse", "HEAD"])
    .stdout.toString()
    .trim();
  return { plugin, revision };
}

function registerPlugin(
  fixture: DeploymentFixture,
  plugin: string,
  revision: string,
): string {
  const registration = join(fixture.root, "registration.json");
  const source = {
    kind: "github",
    owner: "aigorahub",
    repo: "herdr-lantern",
    resolved_commit: revision,
  };
  const entry = { plugin_id: "aigora.lantern", plugin_root: plugin, source };
  writeFileSync(registration, JSON.stringify({ result: { plugins: [entry] } }));
  writeFileSync(
    join(fixture.bin, "herdr"),
    `#!${process.execPath}\nprocess.stdout.write(await Bun.file(${JSON.stringify(registration)}).text());\n`,
  );
  chmodSync(join(fixture.bin, "herdr"), executableMode);
  return registration;
}

function prepareLanternFixture(fixture: DeploymentFixture): LanternFixture {
  const { plugin, revision } = createPlugin(fixture.root);
  mkdirSync(join(fixture.repository, "harness"), { recursive: true });
  writeFileSync(
    join(fixture.repository, "harness/lantern-revision"),
    `${revision}\n`,
  );
  writeFileSync(
    join(fixture.repository, "harness/lantern.md"),
    "# Preferences\nUse dedicated worktrees.\n",
  );
  const registration = registerPlugin(fixture, plugin, revision);
  return {
    ...fixture,
    plugin,
    registration,
    destination: join(
      fixture.home,
      ".config/herdr/plugins/config/aigora.lantern/prompt.md",
    ),
    generated: join(fixture.repository, "harness/generated/lantern/prompt.md"),
  };
}

function runLantern(fixture: DeploymentFixture): CommandResult {
  return runDeploymentHelper(
    fixture,
    {
      helper: "deploy-lantern.ts",
      arguments: [fixture.repository, fixture.home],
    },
    { PATH: `${fixture.bin}:${process.env.PATH ?? ""}` },
  );
}

export { prepareLanternFixture, runLantern };
