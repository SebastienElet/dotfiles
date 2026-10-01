import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  pathExists,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  prepareWorkerCleanup,
  rememWorker,
} from "./clean-deployment-worker.ts";
import { cleanRoots } from "./clean-deployment-paths.ts";
import { randomUUID } from "node:crypto";
import { z } from "zod";

const nativeTimeoutMilliseconds = 15_000;
const testTimeoutMilliseconds = 30_000;
const executableMode = 0o755;
const pollIntervalMilliseconds = 25;
const markerTimeoutMilliseconds = 5000;
const stateSchema = z
  .object({
    pid: z.number().int().positive(),
    stopped: z.boolean(),
    configPresent: z.boolean().optional(),
    plistPresent: z.boolean().optional(),
  })
  .readonly();
setDefaultTimeout(testTimeoutMilliseconds);
afterEach(cleanupDeploymentFixtures);

function native(command: readonly string[]): string {
  const result = Bun.spawnSync([...command], {
    stdout: "pipe",
    stderr: "pipe",
    timeout: nativeTimeoutMilliseconds,
  });
  if (!result.success) {
    throw new Error(
      `native command failed: ${command[0]} status ${result.exitCode}: ${result.stderr.toString()}`,
    );
  }
  return result.stdout.toString();
}

function inspection(label: string): "absent" | "loaded" {
  const output = native([
    "/usr/bin/osascript",
    "-l",
    "JavaScript",
    join(import.meta.dir, "clean-deployment-worker-inspect.js"),
    label,
  ]);
  return z
    .object({ state: z.enum(["absent", "loaded"]) })
    .parse(JSON.parse(output)).state;
}

async function waitForMarker(
  path: string,
): Promise<z.infer<typeof stateSchema>> {
  const deadline = Date.now() + markerTimeoutMilliseconds;
  while (Date.now() < deadline) {
    if (pathExists(path)) {
      return stateSchema.parse(JSON.parse(readFileSync(path, "utf8")));
    }
    await Bun.sleep(pollIntervalMilliseconds);
  }
  throw new Error("native fixture worker did not start");
}

function createNativeWorker(): Readonly<{
  roots: ReturnType<typeof cleanRoots>;
  worker: typeof rememWorker;
  plist: string;
  marker: string;
  config: string;
}> {
  const fixture = createDeploymentFixture("native-remem-worker");
  const roots = cleanRoots(fixture.repository, fixture.home);
  const label = `dev.dotfiles.remem-retirement.${randomUUID()}`;
  const worker = {
    ...rememWorker,
    label,
    definition: (home: string): Readonly<Record<string, unknown>> => ({
      ...rememWorker.definition(home),
      Label: label,
    }),
  };
  const binary = join(roots.home, ".local/bin/remem");
  const source = join(roots.repository, "home/.remem/config.toml");
  const config = join(roots.home, ".remem/config.toml");
  const plist = join(roots.home, "Library/LaunchAgents", `${label}.plist`);
  for (const path of [binary, source, config, plist]) {
    mkdirSync(dirname(path), { recursive: true });
  }
  writeFileSync(source, "capture config\n");
  symlinkSync(source, config);
  writeFileSync(
    binary,
    `#!${process.execPath}\nimport ${JSON.stringify(join(import.meta.dir, "clean-deployment-worker-native-fixture.ts"))};\n`,
  );
  chmodSync(binary, executableMode);
  writeFileSync(
    plist,
    JSON.stringify(worker.definition(realpathSync(roots.home))),
  );
  writeFileSync(
    join(roots.home, "worker-native-input.json"),
    JSON.stringify({ plist }),
  );
  native(["/usr/bin/plutil", "-convert", "xml1", "--", plist]);
  return {
    roots,
    worker,
    plist,
    marker: join(roots.home, "worker-native-state.json"),
    config,
  };
}

function expectStoppedWorker(
  context: ReturnType<typeof createNativeWorker>,
  pid: number,
): void {
  const stopped = stateSchema.parse(
    JSON.parse(readFileSync(context.marker, "utf8")),
  );
  expect(stopped).toMatchObject({
    pid,
    stopped: true,
    configPresent: true,
    plistPresent: true,
  });
  try {
    process.kill(pid, 0);
    throw new Error("native fixture process remains alive");
  } catch (error) {
    expect(z.object({ code: z.literal("ESRCH") }).parse(error).code).toBe(
      "ESRCH",
    );
  }
  expect(pathExists(context.plist)).toBeFalse();
  expect(inspection(context.worker.label)).toBe("absent");
}

test("native launchd capture has stopped before owned plist removal and replays without restarting", async () => {
  const context = createNativeWorker();
  expect(inspection(context.worker.label)).toBe("absent");
  const target = `gui/${process.getuid?.()}/${context.worker.label}`;
  let loaded = false;
  try {
    native([
      "/bin/launchctl",
      "bootstrap",
      `gui/${process.getuid?.()}`,
      context.plist,
    ]);
    loaded = true;
    const running = await waitForMarker(context.marker);
    expect(running.stopped).toBeFalse();
    expect(inspection(context.worker.label)).toBe("loaded");
    const cleanup = prepareWorkerCleanup(context.roots, {
      worker: context.worker,
    });
    expect(cleanup).toBeDefined();
    cleanup?.(true);
    expectStoppedWorker(context, running.pid);
    loaded = false;
    unlinkSync(context.config);
    expect(
      prepareWorkerCleanup(context.roots, { worker: context.worker }),
    ).toBeUndefined();
  } finally {
    if (loaded) {
      native(["/bin/launchctl", "bootout", "--wait", target]);
    }
  }
});
