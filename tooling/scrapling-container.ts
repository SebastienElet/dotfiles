import {
  type Configuration,
  type Container,
  LifecycleError,
  arraysEqual,
  commandError,
  findContainer,
  inspectDockerImage,
  requireDocker,
  requireSuccess,
  runDocker,
} from "./docker-container.ts";
import { dockerImages, imageReferenceSchema } from "./docker-image.ts";

type PreparedConfiguration = Configuration & Readonly<{ containerId: string }>;

const profileVolume = "scrapling-profiles";
const usageFailureExitCode = 64;
const configurationFailureExitCode = 78;
const maximumTimeoutMilliseconds = 300_000;

async function prepareScraplingContainer(
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<PreparedConfiguration> {
  const configuration = readConfiguration(environment);
  await requireDocker(configuration.timeoutMilliseconds);
  const existing = await findContainer(configuration);
  const container = existing ?? (await createContainer(configuration));
  if (existing !== undefined) {
    await reuseContainer(existing, configuration);
  }
  return { ...configuration, containerId: container.Id };
}

function readConfiguration(
  environment: Readonly<NodeJS.ProcessEnv>,
): Configuration {
  const container = environment.SCRAPLING_CONTAINER ?? "scrapling-mcp";
  const image = environment.SCRAPLING_IMAGE ?? dockerImages.scrapling;
  const timeout = environment.SCRAPLING_DOCKER_TIMEOUT_MS ?? "10000";
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]*$/u.test(container)) {
    throw new LifecycleError(
      usageFailureExitCode,
      `invalid container name: ${container}`,
    );
  }
  const parsedImage = imageReferenceSchema.safeParse(image);
  if (!parsedImage.success) {
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

async function reuseContainer(
  container: Container,
  configuration: Configuration,
): Promise<void> {
  const expectedImageId = await inspectDockerImage(configuration);
  if (
    container.Image !== expectedImageId ||
    !isCompatible(container, configuration)
  ) {
    throw new LifecycleError(
      configurationFailureExitCode,
      `container ${configuration.container} is incompatible with the required Scrapling configuration`,
    );
  }
  if (container.State.Running) {
    return;
  }
  const started = await runDocker(
    ["start", container.Id],
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

async function createContainer(
  configuration: Configuration,
): Promise<Container> {
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
  if (created.timedOut) {
    throw commandError(
      1,
      `cannot create container ${configuration.container}`,
      created,
    );
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
  return racedContainer;
}

export { prepareScraplingContainer };
export { LifecycleError } from "./docker-container.ts";
export type { PreparedConfiguration };
