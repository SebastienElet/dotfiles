import {
  type Configuration,
  LifecycleError,
  prepareScraplingContainer,
} from "./scrapling-container.ts";

async function runScraplingMcp(
  environment: Readonly<NodeJS.ProcessEnv>,
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
    { stderr: "inherit", stdin: "inherit", stdout: "inherit" },
  );
  const forwardInterrupt = (): void => {
    child.kill("SIGINT");
  };
  const forwardTermination = (): void => {
    child.kill("SIGTERM");
  };
  process.on("SIGINT", forwardInterrupt);
  process.on("SIGTERM", forwardTermination);
  try {
    return await child.exited;
  } finally {
    process.off("SIGINT", forwardInterrupt);
    process.off("SIGTERM", forwardTermination);
  }
}

export { runScraplingMcp };
