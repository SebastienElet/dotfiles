import { z } from "zod";

const successExitCode = 0;
const dockerTimeoutMilliseconds = 120_000;
const legacyImageReferences = new Set([
  "ghcr.io/firecrawl/firecrawl:latest",
  "ghcr.io/firecrawl/nuq-postgres:latest",
  "ghcr.io/firecrawl/playwright-service:latest",
  "rabbitmq:3-management",
  "redis:alpine",
]);
const containerIdentifierSchema = z
  .string()
  .regex(/^[0-9a-f]{12,64}$/u, "invalid Docker container evidence");
const volumeNameSchema = z
  .string()
  .regex(/^[a-zA-Z0-9][a-zA-Z0-9_.-]+$/u, "invalid Docker volume evidence");
type ContainerEvidence = Readonly<{
  Config: Readonly<{ Image: string }>;
  Id: string;
  Mounts: readonly Readonly<{
    Name?: string | undefined;
    Type: string;
  }>[];
}>;
const containerSchema: z.ZodType<ContainerEvidence> = z.object({
  Config: z.object({ Image: z.string().min(1) }).loose(),
  Id: containerIdentifierSchema,
  Mounts: z.array(
    z
      .object({
        Name: volumeNameSchema.optional(),
        Type: z.string(),
      })
      .loose(),
  ),
});
const containerListSchema: z.ZodType<readonly ContainerEvidence[]> =
  z.array(containerSchema);
const policySchema = z.enum(["allow-skip", "require-docker"]);

type DockerInspection = Readonly<{
  containers: readonly string[];
  images: readonly string[];
  labelledVolumes: readonly string[];
  state: "available";
  volumes: readonly string[];
}>;
type DockerState = DockerInspection | Readonly<{ state: "skipped" }>;

function inspectFirecrawlDocker(
  binary: string,
  policyValue: string,
): DockerState {
  const policy = policySchema.parse(policyValue);
  const daemon = runDocker(binary, ["info"]);
  if (daemon.exitCode !== successExitCode || daemon.timedOut) {
    if (policy === "allow-skip") {
      return { state: "skipped" };
    }
    throw new Error("Docker daemon unavailable");
  }
  const containerIds = findContainerIds(binary);
  const containers = inspectContainers(binary, containerIds);
  const labelledVolumes = findLabelledVolumes(binary);
  const images = findLegacyImages(binary);
  return summarizeDocker({ containerIds, containers, images, labelledVolumes });
}

function summarizeDocker(
  input: Readonly<{
    containerIds: readonly string[];
    containers: readonly ContainerEvidence[];
    images: readonly string[];
    labelledVolumes: readonly string[];
  }>,
): DockerInspection {
  return {
    containers: input.containerIds,
    images: input.images,
    labelledVolumes: input.labelledVolumes,
    state: "available",
    volumes: [
      ...new Set([
        ...input.containers.flatMap((container) =>
          container.Mounts.flatMap((mount) =>
            mount.Type === "volume" && mount.Name !== undefined
              ? [mount.Name]
              : [],
          ),
        ),
        ...input.labelledVolumes,
      ]),
    ].toSorted(),
  };
}

function findLegacyImages(binary: string): readonly string[] {
  const output = requireDocker(
    runDocker(binary, ["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"]),
    "image discovery",
  );
  return [
    ...new Set(
      parseLines(output, (line) => z.string().min(1).parse(line)).filter(
        (reference) => legacyImageReferences.has(reference),
      ),
    ),
  ].toSorted();
}

function findContainerIds(binary: string): readonly string[] {
  const output = requireDocker(
    runDocker(binary, [
      "container",
      "ls",
      "--all",
      "--quiet",
      "--filter",
      "label=com.docker.compose.project=firecrawl",
    ]),
    "container discovery",
  );
  return parseLines(output, (line) => containerIdentifierSchema.parse(line));
}

function findLabelledVolumes(binary: string): readonly string[] {
  const output = requireDocker(
    runDocker(binary, [
      "volume",
      "ls",
      "--quiet",
      "--filter",
      "label=com.docker.compose.project=firecrawl",
    ]),
    "volume discovery",
  );
  return parseLines(output, (line) => volumeNameSchema.parse(line));
}

function inspectContainers(
  binary: string,
  identifiers: readonly string[],
): readonly ContainerEvidence[] {
  if (identifiers.length === 0) {
    return [];
  }
  const output = requireDocker(
    runDocker(binary, ["container", "inspect", "--", ...identifiers]),
    "container inspection",
  );
  try {
    return containerListSchema.parse(JSON.parse(output));
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new TypeError("invalid Docker container evidence", {
        cause: error,
      });
    }
    throw error;
  }
}

function removeFirecrawlDocker(
  binary: string,
  inspection: Readonly<DockerInspection>,
): void {
  if (inspection.containers.length > 0) {
    requireDocker(
      runDocker(binary, [
        "container",
        "rm",
        "--force",
        "--volumes",
        "--",
        ...inspection.containers,
      ]),
      "container removal",
    );
  }
  if (inspection.labelledVolumes.length > 0) {
    requireDocker(
      runDocker(binary, ["volume", "rm", "--", ...inspection.labelledVolumes]),
      "volume removal",
    );
  }
}

function parseLines<Value>(
  output: string,
  parser: (input: string) => Value,
): readonly Value[] {
  return output
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => parser(line));
}

function runDocker(
  binary: string,
  arguments_: readonly string[],
): Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
  timedOut: boolean;
}> {
  const result = Bun.spawnSync([binary, ...arguments_], {
    stderr: "pipe",
    stdout: "pipe",
    timeout: dockerTimeoutMilliseconds,
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
    timedOut: result.exitedDueToTimeout === true,
  };
}

function requireDocker(
  result: ReturnType<typeof runDocker>,
  operation: string,
): string {
  if (result.timedOut || result.exitCode !== successExitCode) {
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new Error(`Docker ${operation} failed`);
  }
  return result.stdout;
}

export { type DockerInspection, inspectFirecrawlDocker, removeFirecrawlDocker };
