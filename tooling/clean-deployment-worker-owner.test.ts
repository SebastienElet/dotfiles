import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  pathExists,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { lstatSync, readFileSync, writeFileSync } from "node:fs";
import {
  workerFixture,
  workerPlist,
  workerState,
} from "./clean-deployment-worker-test-support.ts";

afterEach(cleanupDeploymentFixtures);

test("refuses a substituted foreign caller UID before mutating MCP, config or deployments", () => {
  const context = workerFixture();
  const preload = join(context.root, "foreign-caller.ts");
  const foreignUid = lstatSync(context.home).uid + 1;
  writeFileSync(preload, `process.getuid = () => ${foreignUid};\n`);
  const shared = join(context.home, ".claude.json");
  const mcpCommand =
    'export PATH="$HOME/.local/bin:$HOME/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"; exec "$HOME/.local/bin/remem" mcp';
  const sharedOriginal = JSON.stringify({
    theme: "preserve",
    mcpServers: { remem: { command: "/bin/sh", args: ["-c", mcpCommand] } },
  });
  writeFileSync(shared, sharedOriginal);
  const original = readFileSync(workerPlist(context), "utf8");
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--preload",
      preload,
      join(import.meta.dir, "clean-deployment.ts"),
      context.repository,
      context.home,
      "--apply",
    ],
    {
      env: {
        ...process.env,
        HOME: context.home,
        PATH: `${context.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
        CLEAN_WORKER_TEST_STATE: join(context.root, "worker-state.json"),
      },
    },
  );
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain(
    "Worker HOME is owned by another UID",
  );
  expect(readFileSync(workerPlist(context), "utf8")).toBe(original);
  expect(readFileSync(shared, "utf8")).toBe(sharedOriginal);
  expect(pathExists(join(context.home, ".remem/config.toml"))).toBeTrue();
  expect(workerState(context).trace).toEqual([]);
});
