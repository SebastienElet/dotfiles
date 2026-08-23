import {
  LifecycleError,
  prepareScraplingContainer,
  type Configuration,
} from "./scrapling-container.ts";

export async function runScraplingMcp(
  environment: NodeJS.ProcessEnv,
): Promise<number> {
  try {
    return await runMcp(await prepareScraplingContainer(environment));
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    return error instanceof LifecycleError ? error.exitCode : 1;
  }
}

async function runMcp(configuration: Configuration): Promise<number> {
  const child = Bun.spawn(
    [
      "docker",
      "exec",
      "--interactive",
      configuration.container,
      "uv",
      "run",
      "scrapling",
      "mcp",
    ],
    { stdin: "inherit", stdout: "inherit", stderr: "inherit" },
  );
  const forwardInterrupt = () => child.kill("SIGINT");
  const forwardTermination = () => child.kill("SIGTERM");
  process.on("SIGINT", forwardInterrupt);
  process.on("SIGTERM", forwardTermination);
  try {
    return await child.exited;
  } finally {
    process.off("SIGINT", forwardInterrupt);
    process.off("SIGTERM", forwardTermination);
  }
}
