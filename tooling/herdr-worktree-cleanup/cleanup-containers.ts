import { z } from "zod";

const containerIdsSchema = z.array(z.string().regex(/^[0-9a-f]{12,64}$/u));
const commandTimeoutMilliseconds = 15_000;
type DockerCommand = (
  arguments_: readonly string[],
) => Readonly<{ exitCode: number; stdout: string }>;

function runDocker(arguments_: readonly string[]): ReturnType<DockerCommand> {
  const result = Bun.spawnSync(["docker", ...arguments_], {
    stdout: "pipe",
    stderr: "ignore",
    timeout: commandTimeoutMilliseconds,
    killSignal: "SIGKILL",
  });
  return { exitCode: result.exitCode, stdout: result.stdout.toString() };
}

function cleanupWorktreeContainers(
  checkoutPath: string,
  mode: "off" | "stop" = "off",
  command: DockerCommand = runDocker,
): boolean {
  if (mode === "off") {
    return false;
  }
  try {
    const listed = command([
      "ps",
      "-q",
      "--filter",
      `label=com.docker.compose.project.working_dir=${checkoutPath}`,
    ]);
    if (listed.exitCode !== 0 || listed.stdout.trim() === "") {
      return false;
    }
    const ids = containerIdsSchema.parse(listed.stdout.trim().split("\n"));
    return command(["stop", "--timeout", "10", ...ids]).exitCode === 0;
  } catch {
    return false;
  }
}

export { cleanupWorktreeContainers };
export type { DockerCommand };
