import { z } from "zod";

const successExitCode = 0;
const failureExitCode = 1;
const dockerDaemonProbeTimeoutMilliseconds = 10_000;
const dockerCommandTimeoutMilliseconds = 600_000;
const policySchema = z.enum(["allow-skip", "require-docker"]);
const targetSchema = z.enum(["cloakbrowser", "scrapling"]);
const actionSchema = z.enum(["install", "verify"]);
const imageReferenceSchema = z
  .string()
  .min(1)
  .regex(/^(?!-)(?!.*@)\S+$/u, "unsupported Docker image reference");
const invocationSchema = z.tuple([
  actionSchema,
  targetSchema,
  policySchema,
  imageReferenceSchema,
]);
const imageIdentifierSchema = z.string().regex(/^sha256:[0-9a-f]{64}$/u);
const decoder = new TextDecoder("utf-8", { fatal: true });

type DockerCommandResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
  timedOut: boolean;
}>;
type DockerInstallResult =
  | Readonly<{ result: "skipped"; target: z.infer<typeof targetSchema> }>
  | Readonly<{ result: "verified"; target: z.infer<typeof targetSchema> }>;
type Invocation = z.infer<typeof invocationSchema>;

function runDockerArtifactInstallation(arguments_: readonly string[]): number {
  try {
    const invocation = invocationSchema.parse(arguments_);
    const result = installOrVerifyDockerArtifact(invocation);
    process.stdout.write(
      `docker-install target=${result.target} result=${result.result}${result.result === "skipped" ? " policy=allow-skip" : ""}\n`,
    );
    return successExitCode;
  } catch (error) {
    process.stderr.write(`docker-install: ${renderError(error)}\n`);
    return failureExitCode;
  }
}

function installOrVerifyDockerArtifact(
  invocation: Readonly<Invocation>,
): DockerInstallResult {
  const [, target, policy] = invocation;
  if (Bun.which("docker") === null) {
    throw new Error("Docker CLI unavailable");
  }
  const daemon = runDocker(["info"], dockerDaemonProbeTimeoutMilliseconds);
  if (daemon.timedOut || daemon.exitCode !== successExitCode) {
    forwardOutput(daemon);
    if (policy === "allow-skip") {
      return { result: "skipped", target };
    }
    throw new Error("Docker daemon unavailable and policy requires Docker");
  }
  if (invocation[0] === "install") {
    installDockerArtifact(invocation);
  }
  verifyDockerArtifact(invocation);
  return { result: "verified", target };
}

function installDockerArtifact(invocation: Readonly<Invocation>): void {
  const [_action, _target, _policy, artifact] = invocation;
  if (localImageExists(artifact)) {
    return;
  }
  const command = ["pull", "--", artifact];
  const result = runDocker(command);
  if (result.exitCode === successExitCode && !result.timedOut) {
    forwardOutput(result);
  }
  requireSuccessfulCommand(result, command);
}

function localImageExists(image: string): boolean {
  const command = [
    "image",
    "ls",
    "--quiet",
    "--no-trunc",
    "--filter",
    `reference=${image}`,
  ];
  const result = runDocker(command);
  requireSuccessfulCommand(result, command);
  const identifiers = result.stdout
    .split("\n")
    .map((identifier) => identifier.trim())
    .filter((identifier) => identifier.length > 0);
  return z.array(imageIdentifierSchema).parse(identifiers).length > 0;
}

function verifyDockerArtifact(invocation: Readonly<Invocation>): void {
  const [_action, _target, _policy, artifact] = invocation;
  requireSuccessfulCommand(runDocker(["image", "inspect", "--", artifact]), [
    "image",
    "inspect",
    "--",
    artifact,
  ]);
}

function runDocker(
  arguments_: readonly string[],
  timeout = dockerCommandTimeoutMilliseconds,
): DockerCommandResult {
  const result = Bun.spawnSync(["docker", ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
    timeout,
  });
  return {
    exitCode: result.exitCode,
    stderr: decoder.decode(result.stderr),
    stdout: decoder.decode(result.stdout),
    timedOut: result.exitedDueToTimeout === true,
  };
}

function requireSuccessfulCommand(
  result: DockerCommandResult,
  command: readonly string[],
): void {
  if (result.timedOut) {
    forwardOutput(result);
    throw new Error(`docker ${command.join(" ")} timed out`);
  }
  if (result.exitCode !== successExitCode) {
    forwardOutput(result);
    throw new Error(`docker ${command.join(" ")} failed`);
  }
}

function forwardOutput(result: DockerCommandResult): void {
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
}

function renderError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export { runDockerArtifactInstallation };
