import type { Container, Selection } from "./container-selection.ts";
import {
  containerSchema,
  selectWorktreeContainers,
} from "./container-selection.ts";
import { z } from "zod";

const modeSchema = z.enum(["off", "stop", "remove"]);
const contextsSchema = z
  .array(
    z.object({
      Endpoints: z.object({ docker: z.object({ Host: z.string() }) }),
    }),
  )
  .length(1);
const idsSchema = z.array(z.string().regex(/^[0-9a-f]{64}$/u));
const commandTimeoutMilliseconds = 15_000;
const stopTimeoutSeconds = "10";

type DockerResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}>;
type DockerCommand = (arguments_: readonly string[]) => DockerResult;
type CleanupOutcome = Readonly<{
  kind: "completed" | "skipped" | "failed";
  message: string;
}>;

function runDocker(arguments_: readonly string[]): DockerResult {
  const environment = { ...process.env };
  if (arguments_[0] === "--host") {
    delete environment.DOCKER_CONTEXT;
    delete environment.DOCKER_HOST;
    delete environment.DOCKER_TLS_VERIFY;
    delete environment.DOCKER_CERT_PATH;
  }
  const result = Bun.spawnSync(["docker", ...arguments_], {
    env: environment,
    stdout: "pipe",
    stderr: "pipe",
    timeout: commandTimeoutMilliseconds,
    killSignal: "SIGKILL",
  });
  if (result.signalCode !== undefined) {
    throw new Error(
      `Docker command timed out or was killed: ${arguments_.join(" ")}`,
    );
  }
  const decoder = new TextDecoder("utf-8", { fatal: true });
  return {
    exitCode: result.exitCode,
    stdout: decoder.decode(result.stdout),
    stderr: decoder.decode(result.stderr),
  };
}

function requireSuccess(result: DockerResult, action: string): string {
  if (result.exitCode !== 0) {
    throw new Error(
      `${action}: ${result.stderr.trim() || `status ${result.exitCode}`}`,
    );
  }
  return result.stdout;
}

function localHost(command: DockerCommand): string | undefined {
  if (
    command === runDocker &&
    (process.env.DOCKER_HOST ?? "") !== "" &&
    (process.env.DOCKER_CONTEXT ?? "") === ""
  ) {
    return process.env.DOCKER_HOST;
  }
  const result = command(["context", "inspect"]);
  const contexts = contextsSchema.parse(
    JSON.parse(requireSuccess(result, "cannot inspect Docker context")),
  );
  return contexts[0]?.Endpoints.docker.Host;
}

function inspectContainers(command: DockerCommand): readonly Container[] {
  const listed = requireSuccess(
    command(["container", "ls", "--all", "--quiet", "--no-trunc"]),
    "cannot list containers",
  );
  const ids = idsSchema.parse(
    listed.trim() === "" ? [] : listed.trim().split("\n"),
  );
  return ids.flatMap((id) => {
    const result = command(["container", "inspect", id]);
    if (result.exitCode !== 0) {
      const remaining = requireSuccess(
        command(["container", "ls", "--all", "--quiet", "--no-trunc"]),
        "cannot confirm disappeared container",
      );
      if (
        !idsSchema
          .parse(remaining.trim() === "" ? [] : remaining.trim().split("\n"))
          .includes(id)
      ) {
        return [];
      }
    }
    const inspected = z
      .array(containerSchema)
      .length(1)
      .parse(JSON.parse(requireSuccess(result, "cannot inspect container")));
    if (inspected[0]?.Id !== id) {
      throw new Error("inspection returned a different container ID");
    }
    return inspected;
  });
}

function mutateContainer(
  command: DockerCommand,
  arguments_: readonly string[],
  id: string,
): void {
  const result = command(arguments_);
  if (result.exitCode === 0) {
    return;
  }
  const remaining = requireSuccess(
    command(["container", "ls", "--all", "--quiet", "--no-trunc"]),
    "cannot confirm disappeared container",
  );
  if (
    !idsSchema
      .parse(remaining.trim() === "" ? [] : remaining.trim().split("\n"))
      .includes(id)
  ) {
    return;
  }
  requireSuccess(result, "container cleanup failed");
}

function cleanupWorktreeContainers(
  checkoutPath: string,
  mode: string,
  options: Readonly<{
    runDocker?: DockerCommand;
    protectedCheckouts?: readonly string[];
  }> = {},
): CleanupOutcome {
  try {
    const command = options.runDocker ?? runDocker;
    const parsedMode = modeSchema.parse(mode);
    if (parsedMode === "off") {
      return { kind: "skipped", message: "Docker cleanup is off" };
    }
    if (command === runDocker && Bun.which("docker") === null) {
      return { kind: "skipped", message: "Docker CLI unavailable" };
    }
    const host = localHost(command);
    if (host === undefined || !host.startsWith("unix:///")) {
      return {
        kind: "skipped",
        message: "Docker endpoint is remote or ambiguous",
      };
    }
    const localCommand: DockerCommand = (arguments_) =>
      command(["--host", host, ...arguments_]);
    const available = localCommand(["info"]);
    if (available.exitCode !== 0) {
      return {
        kind: "skipped",
        message: `Docker daemon unavailable: ${available.stderr.trim()}`,
      };
    }
    const selection = selectWorktreeContainers(
      inspectContainers(localCommand),
      checkoutPath,
      options.protectedCheckouts,
    );
    cleanSelectedContainers(selection.containers, parsedMode, localCommand);
    return {
      kind: "completed",
      message: cleanupMessage(selection, parsedMode, host),
    };
  } catch (error) {
    return {
      kind: "failed",
      message: error instanceof Error ? error.message : String(error),
    };
  }
}

function cleanSelectedContainers(
  containers: readonly Container[],
  mode: "stop" | "remove",
  command: DockerCommand,
): void {
  for (const container of containers) {
    if (container.State.Running) {
      mutateContainer(
        command,
        ["container", "stop", "--timeout", stopTimeoutSeconds, container.Id],
        container.Id,
      );
    }
    if (mode === "remove") {
      mutateContainer(command, ["container", "rm", container.Id], container.Id);
    }
  }
}

function cleanupMessage(
  selection: Selection,
  mode: "stop" | "remove",
  host: string,
): string {
  const { containers, skipped } = selection;
  const action = mode === "stop" ? "stopped" : "removed";
  const removal =
    mode === "stop" && containers.length > 0
      ? `\nExplicit removal: docker --host '${host.replaceAll("'", String.raw`'\''`)}' container rm ${containers.map((container) => container.Id).join(" ")}`
      : "";
  return `${action} ${containers.length} worktree containers${removal}\n${skipped.join("\n")}`.trim();
}

export { cleanupWorktreeContainers };
export type { DockerCommand, DockerResult };
