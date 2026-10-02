import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { requireCommand } from "./deployment-test-support.ts";
import { tmpdir } from "node:os";

const repositoryRoot = resolve(import.meta.dir, "..");
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
  home: string;
  binaryDirectory: string;
  localBinaryDirectory: string;
  trace: string;
}>;
type MakeArgumentOptions = Readonly<{
  imageOverride: string | undefined;
  policy: string;
}>;
type DockerInstallResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
  trace: string;
  scraplingLinkExists: boolean;
}>;

function runDockerInstallTarget(
  target: DockerInstallTarget,
  scenario: DockerInstallScenario,
  options: DockerInstallOptions = {},
): DockerInstallResult {
  const { dockerProviderAvailable = true, imageOverride, policy } = options;
  const fixture = createDockerInstallFixture(dockerProviderAvailable);
  const result = Bun.spawnSync({
    cmd:
      target === "scrapling"
        ? [
            process.env.DEPLOYMENT_MOON ?? requireCommand("moon"),
            "exec",
            "--quiet",
            "--ignore-ci-checks",
            "--no-actions",
            "repository:scrapling",
          ]
        : makeArguments(target, fixture, {
            imageOverride,
            policy: policy ?? "require-docker",
          }),
    cwd: repositoryRoot,
    env: {
      ...process.env,
      HOME: fixture.home,
      MOON_HOME: process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
      PROTO_HOME:
        process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
      PROTO_OFFLINE: "true",
      DOCKER_UNAVAILABLE_POLICY: policy,
      SCRAPLING_IMAGE: imageOverride,
      DOCKER_INSTALL_TEST_SCENARIO: scenario,
      DOCKER_INSTALL_TEST_STATE: fixture.trace,
      DOCKER_INSTALL_TEST_TARGET: target,
      PATH: fixture.binaryDirectory,
    },
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
    trace: readFileSync(fixture.trace, "utf8"),
    scraplingLinkExists: existsSync(
      join(fixture.home, ".local/bin/scrapling_mcp"),
    ),
  };
}

function createDockerInstallFixture(
  dockerProviderAvailable: boolean,
): DockerInstallFixture {
  const root = mkdtempSync(join(tmpdir(), "docker-install-"));
  const home = join(root, "home");
  const binaryDirectory = join(root, "bin");
  const localBinaryDirectory = join(root, "local-bin");
  const trace = join(root, "docker-trace");
  fixtures.push(root);
  mkdirSync(home);
  mkdirSync(binaryDirectory);
  symlinkSync(process.execPath, join(binaryDirectory, "bun"));
  mkdirSync(localBinaryDirectory);
  writeFileSync(trace, "");
  chmodSync(provider, executableMode);
  symlinkRequiredCommand("git", binaryDirectory);
  symlinkRequiredCommand("bash", binaryDirectory);
  symlinkRequiredCommand("readlink", binaryDirectory);
  symlinkRequiredCommand("uname", binaryDirectory);
  if (dockerProviderAvailable) {
    symlinkSync(provider, join(binaryDirectory, "docker"));
  }
  symlinkSync(
    join(repositoryRoot, "tooling", "scrapling-mcp"),
    join(localBinaryDirectory, "scrapling_mcp"),
  );
  return { home, binaryDirectory, localBinaryDirectory, trace };
}

function symlinkRequiredCommand(command: string, destination: string): void {
  const executable = Bun.which(command);
  if (executable === null) {
    throw new Error(`${command} is unavailable`);
  }
  symlinkSync(executable, join(destination, command));
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
    requireCommand("make"),
    "--no-print-directory",
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
  type DockerInstallResult,
};
