import type {
  CommandResult,
  DeploymentFixture,
} from "./deployment-test-support.ts";
import { project, requireCommand } from "./deployment-test-support.ts";
import { join } from "node:path";

const decoder = new TextDecoder("utf-8", { fatal: true });

function runDeploymentMoon(
  fixture: DeploymentFixture,
  tasks: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv> = {},
): CommandResult {
  const result = Bun.spawnSync(
    [
      process.env.DEPLOYMENT_MOON ?? requireCommand("moon"),
      "exec",
      "--quiet",
      "--ignore-ci-checks",
      "--no-actions",
      "--upstream",
      "none",
      ...tasks,
    ],
    {
      cwd: project,
      env: {
        ...process.env,
        HOME: fixture.home,
        MOON_HOME:
          process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
        PROTO_HOME:
          process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
        ...environment,
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stderr: decoder.decode(result.stderr),
    stdout: decoder.decode(result.stdout),
  };
}

export { runDeploymentMoon };
