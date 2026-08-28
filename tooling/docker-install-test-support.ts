import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";

const repositoryRoot = resolve(import.meta.dir, "..");
const make = Bun.which("make");
const provider = resolve(import.meta.dir, "docker-install-test-provider.ts");
const executableMode = 0o755;
const fixtures: string[] = [];

type DockerInstallScenario =
  | "artifact-absent"
  | "artifact-present"
  | "command-failure"
  | "daemon-unavailable"
  | "invalid-evidence";
type DockerInstallTarget = "cloakbrowser" | "scrapling";
type DockerInstallOptions = Readonly<{
  dockerProviderAvailable?: boolean;
  imageOverride?: string;
  policy?: string;
}>;
type DockerInstallFixture = Readonly<{
  binaryDirectory: string;
  localBinaryDirectory: string;
  makeCommand: string;
  trace: string;
}>;
type MakeArgumentOptions = Readonly<{
  imageOverride: string | undefined;
  policy: string;
}>;
type MakeResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
  trace: string;
}>;

function runDockerInstallTarget(
  target: DockerInstallTarget,
  scenario: DockerInstallScenario,
  options: DockerInstallOptions = {},
): MakeResult {
  if (make === null) {
    throw new Error("make is unavailable");
  }
  const {
    dockerProviderAvailable = true,
    imageOverride,
    policy = "allow-skip",
  } = options;
  const fixture = createDockerInstallFixture(dockerProviderAvailable, make);
  const result = Bun.spawnSync({
    cmd: makeArguments(target, fixture, { imageOverride, policy }),
    cwd: repositoryRoot,
    env: {
      ...process.env,
      DOCKER_INSTALL_TEST_SCENARIO: scenario,
      DOCKER_INSTALL_TEST_STATE: fixture.trace,
      DOCKER_INSTALL_TEST_TARGET: target,
      PATH: `${fixture.binaryDirectory}:${dirname(process.execPath)}:/usr/bin:/bin`,
    },
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
    trace: readFileSync(fixture.trace, "utf8"),
  };
}

function createDockerInstallFixture(
  dockerProviderAvailable: boolean,
  makeCommand: string,
): DockerInstallFixture {
  const root = mkdtempSync(join(tmpdir(), "docker-install-"));
  const binaryDirectory = join(root, "bin");
  const localBinaryDirectory = join(root, "local-bin");
  const trace = join(root, "docker-trace");
  fixtures.push(root);
  mkdirSync(binaryDirectory);
  mkdirSync(localBinaryDirectory);
  writeFileSync(trace, "");
  chmodSync(provider, executableMode);
  if (dockerProviderAvailable) {
    symlinkSync(provider, join(binaryDirectory, "docker"));
  }
  symlinkSync(
    join(repositoryRoot, "tooling", "scrapling-mcp"),
    join(localBinaryDirectory, "scrapling_mcp"),
  );
  return { binaryDirectory, localBinaryDirectory, makeCommand, trace };
}

function makeArguments(
  target: DockerInstallTarget,
  fixture: DockerInstallFixture,
  options: MakeArgumentOptions,
): string[] {
  const imageAssignment =
    options.imageOverride === undefined
      ? []
      : [
          `${target === "scrapling" ? "SCRAPLING_IMAGE" : "CLOAKBROWSER_IMAGE"}=${options.imageOverride}`,
        ];
  return [
    fixture.makeCommand,
    "--no-print-directory",
    "--old-file=docker",
    "--old-file=bun",
    target,
    `LOCAL_BIN=${fixture.localBinaryDirectory}`,
    `DOCKER_UNAVAILABLE_POLICY=${options.policy}`,
    ...imageAssignment,
  ];
}

function cleanupDockerInstallFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture, { force: true, recursive: true });
  }
}

export {
  cleanupDockerInstallFixtures,
  runDockerInstallTarget,
  type DockerInstallScenario,
  type DockerInstallTarget,
  type MakeResult,
};
