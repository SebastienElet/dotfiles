import { parseImageIdentity } from "./docker-image.ts";
import { z } from "zod";

const portBindingSchema = z
  .object({ HostIp: z.string(), HostPort: z.string() })
  .readonly();
const portBindingsSchema = z
  .record(z.string(), z.array(portBindingSchema).readonly().nullable())
  .readonly();
const mountSchema = z
  .object({
    Destination: z.string(),
    Name: z.string().optional(),
    RW: z.boolean(),
    Type: z.string(),
  })
  .readonly();
const containerSchema = z
  .object({
    Id: z.string().regex(/^[0-9a-f]{64}$/u),
    Image: z.string().regex(/^sha256:[0-9a-f]{64}$/u),
    Config: z
      .object({
        Cmd: z.array(z.string()).readonly().nullable(),
        Entrypoint: z.array(z.string()).readonly().nullable(),
        Image: z.string(),
      })
      .readonly(),
    HostConfig: z
      .object({
        ExtraHosts: z.array(z.string()).readonly().nullable(),
        PortBindings: portBindingsSchema.optional(),
      })
      .readonly(),
    Mounts: z.array(mountSchema).readonly(),
    Name: z.string(),
    State: z.object({ Running: z.boolean() }).readonly(),
  })
  .readonly();

type CommandResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}>;

type Container = z.infer<typeof containerSchema>;

type Configuration = Readonly<{
  container: string;
  image: string;
  timeoutMilliseconds: number;
}>;

const decoder = new TextDecoder("utf-8", { fatal: true });
const unavailableFailureExitCode = 69;
const configurationFailureExitCode = 78;
const dataFailureExitCode = 65;
const timeoutFailureExitCode = 75;

class LifecycleError extends Error {
  public readonly exitCode: number;

  public constructor(exitCode: number, message: string) {
    super(message);
    this.exitCode = exitCode;
    this.name = "LifecycleError";
  }
}

async function requireDocker(timeoutMilliseconds: number): Promise<void> {
  if (Bun.which("docker") === null) {
    throw new LifecycleError(
      unavailableFailureExitCode,
      "Docker CLI unavailable",
    );
  }
  const result = await runDocker(["info"], timeoutMilliseconds);
  if (result.timedOut || result.exitCode !== 0) {
    throw commandError(
      unavailableFailureExitCode,
      "Docker daemon unavailable",
      result,
    );
  }
}

async function findContainer(
  configuration: Configuration,
): Promise<Container | undefined> {
  const listed = await runDocker(
    ["container", "ls", "--all", "--format", "{{.Names}}"],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(listed, "cannot list Docker containers");
  const names = listed.stdout.split("\n").filter(Boolean);
  if (!names.includes(configuration.container)) {
    return undefined;
  }
  const inspected = await runDocker(
    ["container", "inspect", "--format", "{{json .}}", configuration.container],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(
    inspected,
    `cannot inspect container ${configuration.container}`,
  );
  try {
    return containerSchema.parse(JSON.parse(inspected.stdout));
  } catch {
    throw new LifecycleError(
      configurationFailureExitCode,
      `container ${configuration.container} returned invalid inspection data`,
    );
  }
}

async function runDocker(
  arguments_: readonly string[],
  timeoutMilliseconds: number,
): Promise<CommandResult> {
  const child = Bun.spawn(["docker", ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
  });
  let timedOut = false;
  const timeout = setTimeout(() => {
    timedOut = true;
    child.kill("SIGKILL");
  }, timeoutMilliseconds);
  const [exitCode, stdoutBytes, stderrBytes] = await Promise.all([
    child.exited,
    new Response(child.stdout).arrayBuffer(),
    new Response(child.stderr).arrayBuffer(),
  ]);
  clearTimeout(timeout);
  try {
    return {
      exitCode,
      stderr: decoder.decode(stderrBytes),
      stdout: decoder.decode(stdoutBytes),
      timedOut,
    };
  } catch {
    throw new LifecycleError(
      dataFailureExitCode,
      `Docker ${arguments_[0] ?? "command"} returned invalid UTF-8`,
    );
  }
}

function requireSuccess(result: CommandResult, action: string): void {
  if (result.timedOut || result.exitCode !== 0) {
    throw commandError(1, action, result);
  }
}

function commandError(
  exitCode: number,
  action: string,
  result: CommandResult,
): LifecycleError {
  if (result.timedOut) {
    return new LifecycleError(timeoutFailureExitCode, `${action} timed out`);
  }
  const detail = result.stderr.trim();
  return new LifecycleError(
    exitCode,
    detail ? `${action}: ${detail}` : `${action} (status ${result.exitCode})`,
  );
}

function arraysEqual(
  actual: readonly string[] | null,
  expected: readonly string[],
): boolean {
  return (
    actual?.length === expected.length &&
    actual.every((value, index) => value === expected[index])
  );
}

async function inspectDockerImage(
  configuration: Configuration,
): Promise<string> {
  const result = await runDocker(
    ["image", "inspect", "--format", "{{json .}}", "--", configuration.image],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(
    result,
    `cannot inspect required image ${configuration.image}; install the pinned image first`,
  );
  try {
    return parseImageIdentity(result.stdout, configuration.image);
  } catch (error) {
    throw new LifecycleError(
      configurationFailureExitCode,
      error instanceof Error ? error.message : String(error),
    );
  }
}

export {
  LifecycleError,
  arraysEqual,
  commandError,
  findContainer,
  inspectDockerImage,
  requireDocker,
  requireSuccess,
  runDocker,
};
export type { Container, Configuration };
