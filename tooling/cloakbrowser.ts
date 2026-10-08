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
import { z } from "zod";

const usageFailureExitCode = 64;
const configurationFailureExitCode = 78;
const maximumTimeoutMilliseconds = 300_000;
const configurationSchema = z.object({
  container: z.string().regex(/^[A-Za-z0-9][A-Za-z0-9_.-]*$/u),
  image: imageReferenceSchema,
  timeoutMilliseconds: z.coerce
    .number()
    .int()
    .positive()
    .max(maximumTimeoutMilliseconds),
});

async function runCloakBrowser(
  environment: Readonly<NodeJS.ProcessEnv>,
): Promise<number> {
  try {
    const parsed = configurationSchema.safeParse({
      container: environment.CLOAKBROWSER_CONTAINER ?? "cloak",
      image: environment.CLOAKBROWSER_IMAGE ?? dockerImages.cloakbrowser,
      timeoutMilliseconds:
        environment.CLOAKBROWSER_DOCKER_TIMEOUT_MS ?? "10000",
    });
    if (!parsed.success) {
      throw new LifecycleError(
        usageFailureExitCode,
        `invalid CloakBrowser configuration: ${parsed.error.message}`,
      );
    }
    await prepareCloakBrowser(parsed.data);
    return 0;
  } catch (error) {
    process.stderr.write(
      `cloakbrowser: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    return error instanceof LifecycleError ? error.exitCode : 1;
  }
}

async function prepareCloakBrowser(
  configuration: Configuration,
): Promise<void> {
  await requireDocker(configuration.timeoutMilliseconds);
  const existing = await findContainer(configuration);
  if (existing !== undefined) {
    await reuseCloakBrowser(existing, configuration);
    return;
  }
  const created = await runDocker(
    [
      "run",
      "--detach",
      "--name",
      configuration.container,
      "--publish",
      "127.0.0.1:9222:9222",
      configuration.image,
      "cloakserve",
      "--idle-timeout=300",
    ],
    configuration.timeoutMilliseconds,
  );
  if (created.timedOut) {
    throw commandError(1, "cannot create CloakBrowser container", created);
  }
  const container = await findContainer(configuration);
  if (container === undefined) {
    throw commandError(1, "cannot create CloakBrowser container", created);
  }
  await reuseCloakBrowser(container, configuration);
}

async function reuseCloakBrowser(
  container: Container,
  configuration: Configuration,
): Promise<void> {
  const expectedImageId = await inspectDockerImage(configuration);
  const ports = container.HostConfig.PortBindings?.["9222/tcp"];
  if (
    container.Name !== `/${configuration.container}` ||
    container.Config.Image !== configuration.image ||
    container.Image !== expectedImageId ||
    !arraysEqual(container.Config.Entrypoint, ["/entrypoint.sh"]) ||
    !arraysEqual(container.Config.Cmd, ["cloakserve", "--idle-timeout=300"]) ||
    ports?.length !== 1 ||
    ports[0]?.HostIp !== "127.0.0.1" ||
    ports[0].HostPort !== "9222"
  ) {
    throw new LifecycleError(
      configurationFailureExitCode,
      `container ${configuration.container} is incompatible with the required CloakBrowser configuration`,
    );
  }
  if (!container.State.Running) {
    const started = await runDocker(
      ["start", container.Id],
      configuration.timeoutMilliseconds,
    );
    requireSuccess(
      started,
      `cannot start container ${configuration.container}`,
    );
  }
}

export { runCloakBrowser };
