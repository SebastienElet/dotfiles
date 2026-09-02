import type {
  MutationManifest,
  MutationWorkflowCoreInput,
} from "./harness-reflection-mutation-workflow-types.ts";
import { join, resolve } from "node:path";
import {
  marginalAblation,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import {
  parseMutationManifest,
  parseMutationWorkflowCoreInput,
} from "./harness-reflection-mutation-workflow-types.ts";
import type { InvariantRecord } from "./invariant-registry-contract.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";
import { tmpdir } from "node:os";

const sourceRepositoryRoot = resolve(import.meta.dir, "..");
const registryPath = "harness/invariants/registry.json";
const surfacePath = "harness/AGENTS.md";
const approvedAt = "2026-09-02T00:00:00.000Z";
const temporaryRoots: string[] = [];

type RegistryPair = Readonly<{
  active: string;
  activeRecord: InvariantRecord;
  retired: string;
  retiredRecord: InvariantRecord;
}>;
type ApprovedManifestOptions = Readonly<{
  registryAfter?: unknown;
  surfaceReplacement?: string;
}>;

const temporaryRoot = async (): Promise<string> => {
  const root = await mkdtemp(join(tmpdir(), "harness-authorization-"));
  temporaryRoots.push(root);
  return root;
};

const supportedRecord = (record: InvariantRecord): InvariantRecord => {
  const { oracle: _oracle, ...recordWithoutOracle } = record;
  const [supported] = parseInvariantRegistry({
    invariants: [
      {
        ...recordWithoutOracle,
        consumers: {
          claude: {
            mechanism: "claude-global-instruction",
            state: "supported",
          },
          codex: {
            mechanism: "codex-global-instruction",
            state: "supported",
          },
          cursor: {
            reason: "No managed instruction surface.",
            state: "unsupported",
          },
        },
        controlKind: "probabilistic",
        marginalAblation,
        surface: "always-loaded-instruction",
        verification: verifiedVerification,
      },
    ],
    version: 1,
  }).invariants;
  if (supported === undefined) {
    throw new Error("supported-active-registry-fixture-empty");
  }
  return supported;
};

const registryPair = async (): Promise<RegistryPair> => {
  const active = await readFile(
    join(
      sourceRepositoryRoot,
      "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
    ),
    "utf8",
  );
  const [activeRecord] = parseInvariantRegistry(JSON.parse(active)).invariants;
  if (activeRecord === undefined) {
    throw new Error("active-registry-fixture-empty");
  }
  const supportedActiveRecord = supportedRecord(activeRecord);
  const retired = JSON.stringify({
    invariants: [
      {
        ...supportedActiveRecord,
        lifecycle: "retired",
        retirement: { reason: "Superseded.", retiredAt: approvedAt },
      },
    ],
    version: 1,
  });
  const [retiredRecord] = parseInvariantRegistry(
    JSON.parse(retired),
  ).invariants;
  if (retiredRecord === undefined) {
    throw new Error("retired-registry-fixture-empty");
  }
  return {
    active: JSON.stringify({ invariants: [supportedActiveRecord], version: 1 }),
    activeRecord: supportedActiveRecord,
    retired,
    retiredRecord,
  };
};

const initializeRepository = async (
  root: string,
  activeRegistry: string,
): Promise<void> => {
  await mkdir(join(root, "harness/invariants"), { recursive: true });
  await mkdir(join(root, "tooling"), { recursive: true });
  await writeFile(join(root, surfacePath), "old surface", "utf8");
  await writeFile(join(root, "unrelated.txt"), "unrelated before", "utf8");
  await writeFile(join(root, registryPath), activeRegistry, "utf8");
  await writeFile(
    join(root, "tooling/git-main-branch-entry.test.ts"),
    "export {};\n",
    "utf8",
  );
  const initialization = Bun.spawnSync(["git", "-C", root, "init", "--quiet"]);
  if (initialization.exitCode !== 0) {
    throw new Error("temporary-git-init-failed");
  }
  const staging = Bun.spawnSync([
    "git",
    "-C",
    root,
    "add",
    "tooling/git-main-branch-entry.test.ts",
  ]);
  if (staging.exitCode !== 0) {
    throw new Error("temporary-git-add-failed");
  }
};

const approvedManifest = (
  pair: RegistryPair,
  registryReplacement: string = pair.retired,
  options: ApprovedManifestOptions = {},
): MutationManifest => {
  const registryAfter = options.registryAfter ?? pair.retiredRecord;
  const surfaceReplacement = options.surfaceReplacement ?? "new surface";
  return parseMutationManifest({
    files: [
      {
        path: surfacePath,
        preimage: "old surface",
        replacement: surfaceReplacement,
      },
      {
        path: registryPath,
        preimage: pair.active,
        replacement: registryReplacement,
      },
    ],
    registryDelta: {
      after: registryAfter,
      before: pair.activeRecord,
      targetInvariantId: "prevent-fetch-url-secret-redaction",
    },
  });
};

const mutationInput = (
  manifest: MutationManifest,
  preparedFiles: readonly Readonly<{ contents: string; path: string }>[],
): MutationWorkflowCoreInput =>
  parseMutationWorkflowCoreInput({
    approval: {
      approvedAt,
      approvedBy: "Sebastien",
      manifest,
    },
    preparedFiles,
    registryPath,
    targetInvariantId: "prevent-fetch-url-secret-redaction",
  });

const cleanupTemporaryRoots = async (): Promise<void> => {
  for (const root of temporaryRoots.splice(0)) {
    await rm(root, { force: true, recursive: true });
  }
};

export {
  approvedAt,
  approvedManifest,
  cleanupTemporaryRoots,
  initializeRepository,
  mutationInput,
  registryPair,
  registryPath,
  surfacePath,
  temporaryRoot,
  type RegistryPair,
};
