import {
  type InvariantRecord,
  type InvariantRegistry,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { join, resolve } from "node:path";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import {
  promotionRequest,
  registryPath,
  retirementApproval,
  surfacePath,
} from "./harness-reflection-mutation-test-support.ts";
import { installFixtureCli } from "./harness-reflection-mutation-integration-cli-support.ts";
import { tmpdir } from "node:os";
import { validateAppliedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const oraclePath =
  "./tooling/invariant-registry-fixtures/.runtime/workflow-state.test.ts";
const oracleSourcePath = resolve(
  import.meta.dir,
  "invariant-registry-fixtures/.runtime/workflow-state.test.ts",
);
const temporaryRoots: string[] = [];
type SyntheticRequest = ReturnType<typeof promotionRequest>;
type SyntheticRequestFile =
  SyntheticRequest["approval"]["manifest"]["files"][number];
type FixtureState = Readonly<{
  registry: InvariantRegistry;
  source: string;
  surface: string;
}>;

const oracle = {
  failurePath: "fixture registry lifecycle disagrees with its surface text",
  invocation: ["bun", "test", oraclePath],
  name: "synthetic-workflow-state",
  testPath: oraclePath,
};

const initializeFixture = async (
  request: Readonly<SyntheticRequest>,
): Promise<string> => {
  const root = await mkdtemp(join(tmpdir(), "invariant-workflow-"));
  temporaryRoots.push(root);
  const [surface, registry] = request.approval.manifest.files;
  if (
    surface === undefined ||
    surface.preimage === null ||
    registry === undefined
  ) {
    throw new Error("fixture-manifest-invalid");
  }
  await mkdir(join(root, "harness/invariants"), { recursive: true });
  await mkdir(join(root, "tooling/invariant-registry-fixtures/.runtime"), {
    recursive: true,
  });
  await writeFile(join(root, surfacePath), surface.preimage, "utf8");
  await writeFile(join(root, registryPath), registry.preimage ?? "", "utf8");
  await writeFile(
    join(root, oraclePath),
    await readFile(oracleSourcePath, "utf8"),
    "utf8",
  );
  await installFixtureCli(root);
  if (Bun.spawnSync(["git", "-C", root, "init", "--quiet"]).exitCode !== 0) {
    throw new Error("fixture-git-init-failed");
  }
  if (Bun.spawnSync(["git", "-C", root, "add", oraclePath]).exitCode !== 0) {
    throw new Error("fixture-git-add-failed");
  }
  return root;
};

const currentFiles = async (
  root: string,
  request: Readonly<SyntheticRequest>,
): Promise<Readonly<Record<string, string>>> => {
  const entries = await Promise.all(
    request.approval.manifest.files.map(
      async ({ path }): Promise<readonly [string, string]> => [
        path,
        await readFile(join(root, path), "utf8"),
      ],
    ),
  );
  return Object.fromEntries(entries);
};

const applySyntheticSurface = async (
  root: string,
  request: Readonly<SyntheticRequest>,
): Promise<void> => {
  const surface = request.approval.manifest.files.find(
    ({ path }) => path !== registryPath,
  );
  if (surface === undefined) {
    throw new Error("fixture-surface-missing");
  }
  await writeFile(join(root, surface.path), surface.replacement, "utf8");
};

const recordSyntheticRegistry = async (
  root: string,
  request: Readonly<SyntheticRequest>,
): Promise<void> => {
  const registry = request.approval.manifest.files.find(
    ({ path }) => path === registryPath,
  );
  if (registry === undefined) {
    throw new Error("fixture-registry-missing");
  }
  await writeFile(join(root, registry.path), registry.replacement, "utf8");
};

const validateAppliedFixture = async (
  root: string,
  request: Readonly<SyntheticRequest>,
): Promise<void> => {
  validateAppliedHarnessMutation(request, await currentFiles(root, request));
};

const readFixtureState = async (root: string): Promise<FixtureState> => {
  const source = await readFile(join(root, registryPath), "utf8");
  const registry = parseInvariantRegistry(JSON.parse(source));
  return {
    registry,
    source,
    surface: await readFile(join(root, surfacePath), "utf8"),
  };
};

const retiredRecord = (active: InvariantRecord): InvariantRecord => {
  const [retired] = parseInvariantRegistry({
    invariants: [
      {
        ...active,
        approval: retirementApproval,
        lifecycle: "retired",
        retirement: {
          reason: "Synthetic workflow retirement.",
          retiredAt: retirementApproval.approvedAt,
        },
      },
    ],
    version: 1,
  }).invariants;
  if (retired === undefined) {
    throw new Error("retired-record-missing");
  }
  return retired;
};

const retirementFiles = (
  promotion: Readonly<SyntheticRequest>,
  active: InvariantRecord,
  retired: InvariantRecord,
): readonly SyntheticRequestFile[] => {
  const activeRegistry = JSON.stringify({ invariants: [active], version: 1 });
  const retiredRegistry = JSON.stringify({ invariants: [retired], version: 1 });
  const activeSurface = promotion.approval.manifest.files.find(
    ({ path }) => path === surfacePath,
  );
  if (activeSurface === undefined) {
    throw new Error("active-surface-missing");
  }
  return [
    {
      path: surfacePath,
      preimage: activeSurface.replacement,
      replacement: "Existing guidance.\n",
    },
    {
      path: registryPath,
      preimage: activeRegistry,
      replacement: retiredRegistry,
    },
  ] as const;
};

const retirementRequestFrom = (
  promotion: Readonly<SyntheticRequest>,
): SyntheticRequest => {
  const active = promotion.approval.manifest.registryDelta.after;
  if (active === null) {
    throw new Error("active-record-missing");
  }
  const retired = retiredRecord(active);
  const files = retirementFiles(promotion, active, retired);
  return {
    approval: {
      ...retirementApproval,
      manifest: {
        files,
        registryDelta: {
          after: retired,
          before: active,
          targetInvariantId: active.id,
        },
      },
    },
    preparedFiles: files.map(
      ({ path, preimage, replacement }: Readonly<(typeof files)[number]>) => ({
        contents: replacement,
        path,
        preimage,
      }),
    ),
    targetInvariantId: active.id,
  };
};

const cleanup = async (): Promise<void> => {
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
};

const syntheticPromotionRequest = (
  sources: InvariantRecord["sources"],
): SyntheticRequest => promotionRequest(sources, undefined, oracle);

export {
  applySyntheticSurface,
  cleanup,
  initializeFixture,
  readFixtureState,
  recordSyntheticRegistry,
  retirementRequestFrom,
  syntheticPromotionRequest,
  validateAppliedFixture,
};
export { runFixtureCli } from "./harness-reflection-mutation-integration-cli-support.ts";
