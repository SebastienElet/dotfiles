import type {
  CommandResult,
  DeploymentFixture,
} from "./deployment-test-support.ts";
import { join } from "node:path";
import { requireCommand } from "./deployment-test-support.ts";
import { withoutMoonTaskContext } from "./deployment-moon-test-support.ts";

const decoder = new TextDecoder("utf-8", { fatal: true });

function runDeploymentMoon(
  fixture: Pick<DeploymentFixture, "home" | "bin" | "workspace">,
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
      cwd: fixture.workspace,
      env: {
        ...withoutMoonTaskContext(process.env),
        HOME: fixture.home,
        PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
        MOON_HOME:
          process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
        PROTO_HOME:
          process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
        CARGO_HOME:
          process.env.CARGO_HOME ?? join(process.env.HOME ?? "", ".cargo"),
        RUSTUP_HOME:
          process.env.RUSTUP_HOME ?? join(process.env.HOME ?? "", ".rustup"),
        ...environment,
        PROTO_OFFLINE: "true",
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
