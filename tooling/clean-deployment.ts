import {
  type Artifact,
  deploymentArtifacts,
} from "./clean-deployment-inventory.ts";
import {
  applyConfiguration,
  configurationUpdates,
} from "./clean-deployment-config.ts";
import {
  canonicalLinkTarget,
  cleanRoots,
  verifyParents,
} from "./clean-deployment-paths.ts";
import { dirname, resolve } from "node:path";
import { lstatSync, readlinkSync, unlinkSync } from "node:fs";
import { gitIncludeUpdates } from "./clean-deployment-git.ts";
import { prepareWorkerCleanup } from "./clean-deployment-worker.ts";
import { z } from "zod";

const pathSchema = z.string().min(1);
const argumentsSchema = z.union([
  z.tuple([pathSchema, pathSchema]),
  z.tuple([pathSchema, pathSchema, z.literal("--apply")]),
]);
const argumentOffset = 2;

function ownedArtifact(artifact: Artifact): "absent" | "owned" | "preserved" {
  const metadata = lstatSync(artifact.destination, { throwIfNoEntry: false });
  if (metadata === undefined) {
    return "absent";
  }
  if (artifact.source === undefined) {
    return metadata.isFile() ? "owned" : "preserved";
  }
  if (!metadata.isSymbolicLink()) {
    return "preserved";
  }
  return canonicalLinkTarget(
    resolve(dirname(artifact.destination), readlinkSync(artifact.destination)),
  ) === canonicalLinkTarget(artifact.source)
    ? "owned"
    : "preserved";
}

function cleanArtifact(
  artifact: Artifact,
  roots: Readonly<{ repository: string; home: string }>,
  apply: boolean,
): void {
  verifyParents(roots.home, roots.repository, artifact.destination);
  const state = ownedArtifact(artifact);
  if (state === "absent") {
    return;
  }
  if (state === "preserved") {
    process.stdout.write(`preserved ${artifact.destination}\n`);
    return;
  }
  if (apply) {
    unlinkSync(artifact.destination);
  }
  process.stdout.write(
    `${apply ? "removed" : "would-remove"} ${artifact.destination}\n`,
  );
}

function executeCleanup(actions: readonly (() => void)[]): void {
  let failures = 0;
  for (const action of actions) {
    try {
      action();
    } catch (error) {
      failures += 1;
      process.stderr.write(
        `Error: ${error instanceof Error ? error.message : String(error)}\n`,
      );
    }
  }
  if (failures > 0) {
    throw new Error(`${failures} deployment cleanup failure(s)`);
  }
}

function cleanDeployment(
  repository: string,
  home: string,
  apply: boolean,
): void {
  const roots = cleanRoots(repository, home);
  const artifacts = deploymentArtifacts(roots.repository, roots.home);
  for (const artifact of artifacts) {
    verifyParents(roots.home, roots.repository, artifact.destination);
    ownedArtifact(artifact);
  }
  const updates = [
    ...configurationUpdates(roots.repository, roots.home, [
      { repository: resolve(repository), home: resolve(home) },
    ]).map((update) => ({
      path: update.path,
      apply: (): void => {
        applyConfiguration(update, roots.repository, roots.home);
      },
    })),
    ...gitIncludeUpdates(roots.repository, roots.home),
  ];
  const worker = prepareWorkerCleanup(roots, { homeAliases: [resolve(home)] });
  const configActions = updates.map((update) => (): void => {
    if (apply) {
      update.apply();
    }
    process.stdout.write(
      `${apply ? "updated" : "would-update"} ${update.path}\n`,
    );
  });
  const artifactActions = artifacts.map((artifact) => (): void => {
    cleanArtifact(artifact, roots, apply);
  });
  worker?.(apply);
  executeCleanup([...configActions, ...artifactActions]);
}

if (import.meta.main) {
  try {
    const [repository, home, apply] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    cleanDeployment(repository, home, apply === "--apply");
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
