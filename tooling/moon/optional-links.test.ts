import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  pathExists,
  project,
} from "../deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  writeFileSync,
} from "node:fs";
import { runDeploymentMoon } from "../deployment-moon-runner.ts";

const psqlrcSource = join(project, "home", ".psqlrc");
const curlHttpFailure = 22;
const primeAgentTask = ["harness:prime-agent"];

afterEach(cleanupDeploymentFixtures);

test("links psqlrc and replays silently", () => {
  const fixture = createDeploymentFixture("optional-postgresql");
  expect(runDeploymentMoon(fixture, ["home:postgresql"]).exitCode).toBe(0);
  const destination = join(fixture.home, ".psqlrc");
  expect(readlinkSync(destination)).toBe(psqlrcSource);
  const before = lstatSync(destination).ino;

  const replay = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(lstatSync(destination).ino).toBe(before);
});

test("refuses a divergent psqlrc without replacing it", () => {
  const fixture = createDeploymentFixture("optional-postgresql-collision");
  const destination = join(fixture.home, ".psqlrc");
  writeFileSync(destination, "personal\n");

  const result = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
});

test.skipIf(process.platform !== "darwin").each([
  ["partial download", curlHttpFailure, 'touch "$HOME/executed"'],
  ["installer failure", 0, 'touch "$HOME/executed"; exit 9'],
  ["missing launcher", 0, 'touch "$HOME/executed"'],
])("Moon Prime Agent rejects %s", (_scenario, downloadStatus, installer) => {
  const fixture = createDeploymentFixture("prime-agent-failure");
  writeFileSync(
    join(fixture.bin, "curl"),
    `#!/bin/sh\nprintf '%s\\n' '${installer}'\nexit ${downloadStatus}\n`,
    { mode: 0o755 },
  );

  const result = runDeploymentMoon(fixture, primeAgentTask, {
    PATH: `${fixture.bin}:/usr/bin:/bin`,
  });

  expect(result.exitCode).not.toBe(0);
  expect(pathExists(join(fixture.home, "executed"))).toBe(downloadStatus === 0);
});

test.skipIf(process.platform !== "darwin")(
  "Moon replaces a broken Prime Agent launcher and preserves it on replay",
  () => {
    const fixture = createDeploymentFixture("prime-agent-replay");
    const launcher = join(fixture.home, ".local/bin/prime-agent");
    mkdirSync(dirname(launcher), { recursive: true });
    writeFileSync(launcher, "#!/bin/sh\nexit 1\n", { mode: 0o755 });
    writeFileSync(
      join(fixture.bin, "curl"),
      `#!/bin/sh
cat <<'INSTALLER'
test "$PRIME_AGENT_RELEASE_CHANNEL" = stable || exit 1
printf '#!/bin/sh\\nexit 0\\n' > "$HOME/.local/bin/prime-agent"
chmod +x "$HOME/.local/bin/prime-agent"
INSTALLER
`,
      { mode: 0o755 },
    );
    const environment = {
      PATH: `${fixture.bin}:/usr/bin:/bin`,
      PRIME_AGENT_RELEASE_CHANNEL: "beta",
    };

    expectSuccess(runDeploymentMoon(fixture, primeAgentTask, environment));
    const before = fileIdentity(launcher);
    writeFileSync(join(fixture.bin, "curl"), "#!/bin/sh\nexit 99\n");

    expectSuccess(runDeploymentMoon(fixture, primeAgentTask, environment));
    expect(fileIdentity(launcher)).toEqual(before);
  },
);
