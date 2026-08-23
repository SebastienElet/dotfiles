import { z } from "zod";

const containerSchema = z.object({
  Name: z.string(),
  Config: z.object({
    Image: z.string(),
    Entrypoint: z.array(z.string()).nullable(),
    Cmd: z.array(z.string()).nullable(),
  }),
  HostConfig: z.object({ ExtraHosts: z.array(z.string()).nullable() }),
  Mounts: z.array(
    z.object({
      Type: z.string(),
      Name: z.string().optional(),
      Destination: z.string(),
      RW: z.boolean(),
    }),
  ),
  State: z.object({ Running: z.boolean() }),
});

type CommandResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}>;

export type Configuration = Readonly<{
  container: string;
  image: string;
  timeoutMilliseconds: number;
}>;

const profileVolume = "scrapling-profiles";
const decoder = new TextDecoder("utf-8", { fatal: true });

export async function prepareScraplingContainer(
  environment: NodeJS.ProcessEnv,
): Promise<Configuration> {
  const configuration = readConfiguration(environment);
  await requireDocker(configuration.timeoutMilliseconds);
  const existing = await findContainer(configuration);
  if (existing) await reuseContainer(existing, configuration);
  else await createContainer(configuration);
  return configuration;
}

function readConfiguration(environment: NodeJS.ProcessEnv): Configuration {
  const container = environment.SCRAPLING_CONTAINER ?? "scrapling-mcp";
  const image = environment.SCRAPLING_IMAGE ?? "pyd4vinci/scrapling";
  const timeout = environment.SCRAPLING_DOCKER_TIMEOUT_MS ?? "10000";
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(container)) {
    throw new LifecycleError(64, `invalid container name: ${container}`);
  }
  if (!image || image.startsWith("-") || /[\x00-\x20\x7f]/.test(image)) {
    throw new LifecycleError(64, `invalid image reference: ${image}`);
  }
  if (
    !/^[1-9]\d*$/.test(timeout) ||
    !Number.isSafeInteger(Number(timeout)) ||
    Number(timeout) > 300_000
  ) {
    throw new LifecycleError(64, `invalid Docker timeout: ${timeout}`);
  }
  return { container, image, timeoutMilliseconds: Number(timeout) };
}

async function requireDocker(timeoutMilliseconds: number): Promise<void> {
  const result = await runDocker(["info"], timeoutMilliseconds);
  if (result.exitCode !== 0)
    throw commandError(69, "Docker daemon unavailable", result);
}

async function findContainer(configuration: Configuration) {
  const listed = await runDocker(
    ["container", "ls", "--all", "--format", "{{.Names}}"],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(listed, "cannot list Docker containers");
  const names = listed.stdout.split("\n").filter(Boolean);
  if (!names.includes(configuration.container)) return undefined;
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
      78,
      `container ${configuration.container} returned invalid inspection data`,
    );
  }
}

async function reuseContainer(
  container: z.infer<typeof containerSchema>,
  configuration: Configuration,
): Promise<void> {
  if (!isCompatible(container, configuration)) {
    throw new LifecycleError(
      78,
      `container ${configuration.container} is incompatible with the required Scrapling configuration`,
    );
  }
  if (container.State.Running) return;
  const started = await runDocker(
    ["start", configuration.container],
    configuration.timeoutMilliseconds,
  );
  requireSuccess(started, `cannot start container ${configuration.container}`);
}

function isCompatible(
  container: z.infer<typeof containerSchema>,
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
  if (created.exitCode === 0) return;
  const racedContainer = await findContainer(configuration);
  if (!racedContainer)
    throw commandError(
      1,
      `cannot create container ${configuration.container}`,
      created,
    );
  await reuseContainer(racedContainer, configuration);
}

async function runDocker(
  arguments_: readonly string[],
  timeoutMilliseconds: number,
): Promise<CommandResult> {
  const child = Bun.spawn(["docker", ...arguments_], {
    stdout: "pipe",
    stderr: "pipe",
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
      stdout: decoder.decode(stdoutBytes),
      stderr: decoder.decode(stderrBytes),
      timedOut,
    };
  } catch {
    throw new LifecycleError(
      65,
      `Docker ${arguments_[0] ?? "command"} returned invalid UTF-8`,
    );
  }
}

function requireSuccess(result: CommandResult, action: string): void {
  if (result.exitCode !== 0) throw commandError(1, action, result);
}

function commandError(
  exitCode: number,
  action: string,
  result: CommandResult,
): LifecycleError {
  if (result.timedOut) return new LifecycleError(75, `${action} timed out`);
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

export class LifecycleError extends Error {
  constructor(
    readonly exitCode: number,
    message: string,
  ) {
    super(message);
  }
}
