import type { Container } from "./container-selection.ts";
import type { DockerCommand } from "./cleanup-containers.ts";

const containerIdLength = 64;

function dockerOutput(
  containers: readonly Container[],
  arguments_: readonly string[],
  host: string,
): string {
  if (arguments_.includes("context")) {
    return JSON.stringify([{ Endpoints: { docker: { Host: host } } }]);
  }
  if (arguments_.includes("ls")) {
    return containers.map((container) => container.Id).join("\n");
  }
  if (arguments_.includes("inspect")) {
    return JSON.stringify([
      containers.find((container) => container.Id === arguments_.at(-1)),
    ]);
  }
  return "";
}

function fixtureContainer(character: string, path: string): Container {
  return {
    Id: character.repeat(containerIdLength),
    Config: {
      Labels: {
        "com.docker.compose.project.working_dir": path,
        "com.docker.compose.project": "same-project",
      },
    },
    Mounts: [],
    State: { Running: true },
  };
}

function memoryDocker(
  initial: readonly Container[],
  options: Readonly<{
    host?: string;
    failure?: string;
    disappear?: string;
  }> = {},
): Readonly<{
  containers: ReadonlyMap<string, Container>;
  runDocker: DockerCommand;
}> {
  const containers = new Map(
    initial.map((container) => [
      container.Id,
      { ...container, State: { ...container.State } },
    ]),
  );
  const runDocker: DockerCommand = (arguments_) => {
    if (options.failure !== undefined && arguments_.includes(options.failure)) {
      return { exitCode: 1, stdout: "", stderr: "fixture failure" };
    }
    const id = arguments_.at(-1) ?? "";
    if (
      options.disappear !== undefined &&
      arguments_.includes(options.disappear)
    ) {
      containers.delete(id);
    }
    if (arguments_.includes("stop") || arguments_.includes("rm")) {
      const container = containers.get(id);
      if (container === undefined) {
        return { exitCode: 1, stdout: "", stderr: "container disappeared" };
      }
      container.State.Running = false;
      if (arguments_.includes("rm")) {
        containers.delete(id);
      }
    }
    return {
      exitCode: 0,
      stdout: dockerOutput(
        [...containers.values()],
        arguments_,
        options.host ?? "unix:///fixture.sock",
      ),
      stderr: "",
    };
  };
  return { containers, runDocker };
}

export { fixtureContainer, memoryDocker };
