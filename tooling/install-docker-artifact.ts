import { z } from "zod";

const successExitCode = 0;
const failureExitCode = 1;
const dockerCommandTimeoutMilliseconds = 600_000;
const policySchema = z.enum(["allow-skip", "require-docker"]);
const targetSchema = z.enum(["cloakbrowser", "firecrawl", "scrapling"]);
const actionSchema = z.enum(["install", "verify"]);
const imageReferenceSchema = z
  .string()
  .min(1)
  .regex(/^(?!-)\S+$/u, "invalid Docker image reference");
const invocationSchema = z.union([
  z.tuple([
    actionSchema,
    z.literal("firecrawl"),
    policySchema,
    z.string().min(1),
  ]),
  z.tuple([
    actionSchema,
    z.enum(["cloakbrowser", "scrapling"]),
    policySchema,
    imageReferenceSchema,
  ]),
]);
const serviceNameSchema = z.string().regex(/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/u);
const serviceListSchema = z.array(serviceNameSchema).min(1);
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
  const daemon = runDocker(["info"]);
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
  const [, target, , artifact] = invocation;
  const command =
    target === "firecrawl"
      ? ["compose", "-f", artifact, "up", "--wait", "--wait-timeout", "120"]
      : ["pull", "--", artifact];
  const result = runDocker(command);
  if (result.exitCode === successExitCode && !result.timedOut) {
    forwardOutput(result);
  }
  requireSuccessfulCommand(result, command);
}

function verifyDockerArtifact(invocation: Readonly<Invocation>): void {
  const [, target, , artifact] = invocation;
  if (target !== "firecrawl") {
    requireSuccessfulCommand(runDocker(["image", "inspect", "--", artifact]), [
      "image",
      "inspect",
      "--",
      artifact,
    ]);
    return;
  }
  verifyComposeServices(artifact);
}

function verifyComposeServices(composePath: string): void {
  const prefix = ["compose", "-f", composePath];
  const configured = runDocker([...prefix, "config", "--services"]);
  requireSuccessfulCommand(configured, [...prefix, "config", "--services"]);
  const running = runDocker([
    ...prefix,
    "ps",
    "--services",
    "--status",
    "running",
  ]);
  requireSuccessfulCommand(running, [
    ...prefix,
    "ps",
    "--services",
    "--status",
    "running",
  ]);
  const configuredServices = parseServiceList(configured.stdout);
  const runningServices = new Set(parseServiceList(running.stdout));
  const missingServices = configuredServices.filter(
    (service) => !runningServices.has(service),
  );
  if (missingServices.length > 0) {
    throw new Error(`Docker services absent: ${missingServices.join(", ")}`);
  }
}

function parseServiceList(output: string): readonly string[] {
  return serviceListSchema.parse(
    output
      .split("\n")
      .map((service) => service.trim())
      .filter((service) => service.length > 0),
  );
}

function runDocker(arguments_: readonly string[]): DockerCommandResult {
  const result = Bun.spawnSync(["docker", ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
    timeout: dockerCommandTimeoutMilliseconds,
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
