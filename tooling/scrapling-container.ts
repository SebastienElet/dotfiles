import { z } from "zod";

const containerSchema = z.object({
  Config: z.object({
    Cmd: z.array(z.string()).nullable(),
    Entrypoint: z.array(z.string()).nullable(),
    Image: z.string(),
  }),
  HostConfig: z.object({ ExtraHosts: z.array(z.string()).nullable() }),
  Mounts: z.array(
    z.object({
      Destination: z.string(),
      Name: z.string().optional(),
      RW: z.boolean(),
      Type: z.string(),
    }),
  ),
  Name: z.string(),
  State: z.object({ Running: z.boolean() }),
});

type CommandResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}>;

type Container = Readonly<{
  Config: Readonly<{
    Cmd: readonly string[] | null;
    Entrypoint: readonly string[] | null;
    Image: string;
  }>;
  HostConfig: Readonly<{ ExtraHosts: readonly string[] | null }>;
  Mounts: readonly Readonly<{
    Destination: string;
    Name?: string | undefined;
    RW: boolean;
    Type: string;
  }>[];
  Name: string;
  State: Readonly<{ Running: boolean }>;
}>;

type Configuration = Readonly<{
  container: string;
  image: string;
  timeoutMilliseconds: number;
}>;

const profileVolume = "scrapling-profiles";
const decoder = new TextDecoder("utf-8", { fatal: true });
const usageFailureExitCode = 64;
const unavailableFailureExitCode = 69;
const configurationFailureExitCode = 78;
const dataFailureExitCode = 65;
const timeoutFailureExitCode = 75;
const maximumTimeoutMilliseconds = 300_000;

class LifecycleError extends Error {
  public readonly exitCode: number;

  public constructor(exitCode: number, message: string) {
    super(message);
    this.exitCode = exitCode;
    this.name = "LifecycleError";
  }
}

async function prepareScraplingContainer(
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<Configuration> {
  const configuration = readConfiguration(environment);
  await requireDocker(configuration.timeoutMilliseconds);
  const existing = await findContainer(configuration);
  await (existing === undefined
    ? createContainer(configuration)
    : reuseContainer(existing, configuration));
  return configuration;
}

function readConfiguration(
  environment: Readonly<NodeJS.ProcessEnv>,
): Configuration {
  const container = environment.SCRAPLING_CONTAINER ?? "scrapling-mcp";
  const image = environment.SCRAPLING_IMAGE ?? "pyd4vinci/scrapling";
  const timeout = environment.SCRAPLING_DOCKER_TIMEOUT_MS ?? "10000";
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]*$/u.test(container)) {
    throw new LifecycleError(
      usageFailureExitCode,
      `invalid container name: ${container}`,
    );
  }
  if (!image || image.startsWith("-") || /[\u0000-\u0020\u007F]/u.test(image)) {
    throw new LifecycleError(
      usageFailureExitCode,
      `invalid image reference: ${image}`,
    );
  }
  if (
    !/^[1-9]\d*$/u.test(timeout) ||
    !Number.isSafeInteger(Number(timeout)) ||
    Number(timeout) > maximumTimeoutMilliseconds
  ) {
    throw new LifecycleError(
      usageFailureExitCode,
      `invalid Docker timeout: ${timeout}`,
    );
  }
  return { container, image, timeoutMilliseconds: Number(timeout) };
}

async function requireDocker(timeoutMilliseconds: number): Promise<void> {
  const result = await runDocker(["info"], timeoutMilliseconds);
  if (result.exitCode !== 0) {
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

async function reuseContainer(
  container: Container,
  configuration: Configuration,
): Promise<void> {
  if (!isCompatible(container, configuration)) {
    throw new LifecycleError(
      configurationFailureExitCode,
      `container ${configuration.container} is incompatible with the required Scrapling configuration`,
    );
  }
  if (container.State.Running) {
    return;
  }
  const started = await runDocker(
    ["start", configuration.container],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(started, `cannot start container ${configuration.container}`);
}

function isCompatible(
  container: Container,
  configuration: Configuration,
): boolean {
  const profile = container.Mounts.find(
    (mount) => mount.Destination === "/profiles",
  );
  return (
    container.Name === `/${configuration.container}` &&
    container.Config.Image === configuration.image &&
    arraysEqual(container.Config.Entrypoint, ["sleep"]) &&
    arraysEqual(container.Config.Cmd, ["infinity"]) &&
    (container.HostConfig.ExtraHosts ?? []).includes(
      "host.docker.internal:host-gateway",
    ) &&
    profile?.Type === "volume" &&
    profile.Name === profileVolume &&
    profile.RW
  );
}

async function createContainer(configuration: Configuration): Promise<void> {
  const created = await runDocker(
    [
      "run",
      "--detach",
      "--name",
      configuration.container,
      "--add-host=host.docker.internal:host-gateway",
      "--volume",
      `${profileVolume}:/profiles`,
      "--entrypoint",
      "sleep",
      configuration.image,
      "infinity",
    ],
    configuration.timeoutMilliseconds,
  );
  if (created.exitCode === 0) {
    return;
  }
  const racedContainer = await findContainer(configuration);
  if (!racedContainer) {
    throw commandError(
      1,
      `cannot create container ${configuration.container}`,
      created,
    );
  }
  await reuseContainer(racedContainer, configuration);
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
  if (result.exitCode !== 0) {
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

export { LifecycleError, prepareScraplingContainer };
export type { Configuration };
