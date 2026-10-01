import {
  canonicalLinkTarget,
  verifyParents,
} from "./clean-deployment-paths.ts";
import { dirname, join, resolve } from "node:path";
import { lstatSync, readFileSync, readlinkSync, unlinkSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import { z } from "zod";

const label = "dev.remem.worker";
type WorkerSpecification = Readonly<{
  label: string;
  definition: (home: string) => Readonly<Record<string, unknown>>;
  programArguments: (home: string) => readonly string[];
}>;
const rememWorker: WorkerSpecification = {
  label,
  definition: workerDefinition,
  programArguments: workerArguments,
};

function workerArguments(home: string): readonly string[] {
  return [join(home, ".local/bin/remem"), "worker", "--once"];
}
const nativeTimeoutMilliseconds = 15_000;
const definitionSchema = z
  .object({
    Label: z.string().min(1),
    Program: z.string().min(1).optional(),
    ProgramArguments: z.array(z.string().min(1)).min(1).readonly(),
  })
  .readonly();
const inspectionSchema = z.discriminatedUnion("state", [
  z.object({ state: z.literal("absent") }),
  z.object({ state: z.literal("loaded"), definition: definitionSchema }),
]);
type Roots = Readonly<{ repository: string; home: string }>;
type Inspection = z.infer<typeof inspectionSchema>;
type PreparedWorker = Readonly<{
  target: string;
  path: string;
  homes: readonly string[];
  worker: WorkerSpecification;
  original: string | undefined;
  inspection: Inspection;
}>;

function nativeOutput(command: readonly string[]): string {
  const result = Bun.spawnSync([...command], {
    stdout: "pipe",
    stderr: "pipe",
    timeout: nativeTimeoutMilliseconds,
  });
  if (!result.success) {
    throw new Error(
      `Native worker command ${command[0]} failed (status ${result.exitCode ?? "interrupted"}): ${new TextDecoder("utf-8", { fatal: true }).decode(result.stderr).trim()}`,
    );
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(result.stdout);
}

function inspectWorker(serviceLabel: string): Inspection {
  const output = nativeOutput([
    "osascript",
    "-l",
    "JavaScript",
    join(import.meta.dir, "clean-deployment-worker-inspect.js"),
    serviceLabel,
  ]);
  try {
    return inspectionSchema.parse(JSON.parse(output));
  } catch {
    throw new Error("Native worker inspection returned an unknown shape");
  }
}

function workerDefinition(home: string): Readonly<Record<string, unknown>> {
  return {
    Label: label,
    ProgramArguments: workerArguments(home),
    EnvironmentVariables: {
      PATH: `${home}/.local/bin:${home}/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`,
    },
    RunAtLoad: true,
    StartInterval: 300,
    ProcessType: "Background",
  };
}

function ownsConfiguration(roots: Roots): boolean {
  const path = join(roots.home, ".remem/config.toml");
  verifyParents(roots.home, roots.repository, path);
  if (lstatSync(path, { throwIfNoEntry: false })?.isSymbolicLink() !== true) {
    return false;
  }
  return (
    canonicalLinkTarget(resolve(dirname(path), readlinkSync(path))) ===
    canonicalLinkTarget(join(roots.repository, "home/.remem/config.toml"))
  );
}

function ownedPlist(
  roots: Roots,
  path: string,
  ownership: Pick<PreparedWorker, "homes" | "worker">,
): string | undefined {
  verifyParents(roots.home, roots.repository, path);
  const metadata = lstatSync(path, { throwIfNoEntry: false });
  if (metadata === undefined) {
    return undefined;
  }
  if (!metadata.isFile()) {
    throw new Error(`Preserve foreign worker plist: ${path}`);
  }
  const original = readFileSync(path, "utf8");
  const definition = z
    .record(z.string(), z.unknown())
    .parse(
      JSON.parse(
        nativeOutput(["plutil", "-convert", "json", "-o", "-", "--", path]),
      ),
    );
  if (
    !ownership.homes.some((home) =>
      isDeepStrictEqual(definition, ownership.worker.definition(home)),
    )
  ) {
    throw new Error(`Preserve foreign worker plist: ${path}`);
  }
  if (readFileSync(path, "utf8") !== original) {
    throw new Error(`Worker plist changed during inspection: ${path}`);
  }
  return original;
}

function requireOwnedWorker(
  inspection: Inspection,
  homes: readonly string[],
  worker: WorkerSpecification,
): void {
  if (inspection.state === "absent") {
    return;
  }
  const { definition } = inspection;
  if (
    definition.Label !== worker.label ||
    !homes.some(
      (home) =>
        isDeepStrictEqual(
          definition.ProgramArguments,
          worker.programArguments(home),
        ) &&
        (definition.Program === undefined ||
          definition.Program === join(home, ".local/bin/remem")),
    )
  ) {
    throw new Error("Preserve foreign loaded worker service");
  }
}

function requireUnchangedPlist(
  path: string,
  original: string | undefined,
): void {
  const metadata = lstatSync(path, { throwIfNoEntry: false });
  if (
    original === undefined
      ? metadata !== undefined
      : metadata?.isFile() !== true || readFileSync(path, "utf8") !== original
  ) {
    throw new Error(`Worker plist changed during cleanup: ${path}`);
  }
}

function workerServiceTarget(roots: Roots, serviceLabel: string): string {
  const uid = process.getuid?.();
  if (uid === undefined) {
    throw new Error("Worker service cleanup requires the current user UID");
  }
  if (lstatSync(roots.home).uid !== uid) {
    throw new Error(
      "Worker HOME is owned by another UID; preserve its service and configuration",
    );
  }
  return `gui/${uid}/${serviceLabel}`;
}

function prepareWorkerCleanup(
  roots: Roots,
  options: Readonly<{
    homeAliases?: readonly string[];
    worker?: WorkerSpecification;
  }> = {},
): ((apply: boolean) => void) | undefined {
  const worker = options.worker ?? rememWorker;
  const path = join(
    roots.home,
    "Library/LaunchAgents",
    `${worker.label}.plist`,
  );
  verifyParents(roots.home, roots.repository, path);
  if (
    lstatSync(path, { throwIfNoEntry: false }) === undefined &&
    !ownsConfiguration(roots)
  ) {
    return undefined;
  }
  const target = workerServiceTarget(roots, worker.label);
  const homes = [roots.home, ...(options.homeAliases ?? [])];
  const original = ownedPlist(roots, path, { homes, worker });
  const inspection = inspectWorker(worker.label);
  requireOwnedWorker(inspection, homes, worker);
  const prepared = {
    path,
    homes,
    worker,
    original,
    inspection,
    target,
  };
  return (apply): void => {
    cleanWorker(roots, prepared, apply);
  };
}

function cleanWorker(
  roots: Roots,
  prepared: PreparedWorker,
  apply: boolean,
): void {
  const { path, homes, worker, original, inspection, target } = prepared;
  requireUnchangedPlist(path, original);
  if (!apply) {
    if (inspection.state === "loaded") {
      process.stdout.write(`would-stop ${target}\n`);
    }
    if (original !== undefined) {
      process.stdout.write(`would-remove ${path}\n`);
    }
    return;
  }
  verifyParents(roots.home, roots.repository, path);
  const current = inspectWorker(worker.label);
  requireOwnedWorker(current, homes, worker);
  if (current.state === "loaded") {
    nativeOutput(["launchctl", "bootout", "--wait", target]);
    if (inspectWorker(worker.label).state !== "absent") {
      throw new Error(
        "Worker service remains loaded after bootout; preserve its plist and configuration",
      );
    }
    process.stdout.write(`stopped ${target}\n`);
  }
  requireUnchangedPlist(path, original);
  if (original !== undefined) {
    unlinkSync(path);
    process.stdout.write(`removed ${path}\n`);
  }
}

export { prepareWorkerCleanup, rememWorker };
export type { WorkerSpecification };
