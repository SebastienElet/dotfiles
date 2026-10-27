import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

const executableMode = 0o755;
const moonTestTimeout = 30_000;
afterEach(cleanupDeploymentFixtures);

test.each(["claude", "codex", "cursor"])(
  "Moon installs %s hooks without asking Arnes to write and replays silently",
  (agent) => {
    const fixture = createDeploymentFixture(`hooks-${agent}`);
    for (const name of [
      "arnes",
      "agent-measure",
      "agent-handoff",
      "output-discipline-hook",
    ]) {
      const executable = join(fixture.home, ".local/bin", name);
      mkdirSync(dirname(executable), { recursive: true });
      writeFileSync(
        executable,
        name === "arnes"
          ? '#!/bin/sh\ntest "$1" = doctor\n'
          : "#!/bin/sh\nexit 91\n",
      );
      chmodSync(executable, executableMode);
    }
    const destination = join(
      fixture.home,
      `.${agent}`,
      agent === "claude" ? "settings.json" : "hooks.json",
    );
    const result = runDeploymentMoon(fixture, [`harness:${agent}-hooks`]);
    expectSuccess(result);
    expect(existsSync(destination)).toBeTrue();
    const text = readFileSync(destination, "utf8");
    expect(text).toContain("agent-measure' hook --agent");
    expect(text).not.toContain("/arnes");
    const identity = fileIdentity(destination);
    const replay = runDeploymentMoon(fixture, [`harness:${agent}-hooks`]);
    expectSuccess(replay);
    expect(replay.stdout).toBe("");
    expect(replay.stderr).toBe("");
    expect(fileIdentity(destination)).toEqual(identity);
  },
  moonTestTimeout,
);
