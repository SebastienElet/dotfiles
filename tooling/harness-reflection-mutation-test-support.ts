import {
  type InvariantRecord,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import {
  candidate,
  marginalAblation,
  secondPullRequest,
  source,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";

const registryPath = "harness/invariants/registry.json";
const surfacePath = "harness/AGENTS.md";
const promotionApproval = {
  approvedAt: "2026-09-03T09:00:00.000Z",
  approvedBy: "Reviewer",
};
const retirementApproval = {
  approvedAt: "2026-09-04T09:00:00.000Z",
  approvedBy: "Second reviewer",
};

type RequestFile = Readonly<{
  path: string;
  preimage: string | null;
  replacement: string;
}>;
type ApprovalAttestation = Readonly<{
  approvedAt: string;
  approvedBy: string;
}>;
type ApprovedRequest = Readonly<{
  approval: ApprovalAttestation &
    Readonly<{
      manifest: Readonly<{
        files: readonly RequestFile[];
        registryDelta: Readonly<{
          after: InvariantRecord;
          before: InvariantRecord;
          targetInvariantId: string;
        }>;
      }>;
    }>;
  preparedFiles: readonly Readonly<{
    contents: string;
    path: string;
    preimage: string | null;
  }>[];
  targetInvariantId: string;
}>;
type RecordPair = Readonly<{
  after: InvariantRecord;
  before: InvariantRecord;
}>;
type RetirementPair = Readonly<{
  active: InvariantRecord;
  retired: InvariantRecord;
}>;
type RequestInput = Readonly<{
  after: InvariantRecord;
  approval: ApprovalAttestation;
  before: InvariantRecord;
  files: readonly RequestFile[];
}>;

const defaultPromotionSurface: RequestFile = {
  path: surfacePath,
  preimage: "Existing guidance.\n",
  replacement: `Existing guidance.\n${marginalAblation.candidateTextExact}\n`,
};
const defaultRetirementSurface: RequestFile = {
  path: surfacePath,
  preimage: marginalAblation.candidateTextExact,
  replacement: "Replacement guidance.\n",
};

const record = (value: unknown): InvariantRecord => {
  const [parsed] = parseInvariantRegistry({
    invariants: [value],
    version: 1,
  }).invariants;
  if (parsed === undefined) {
    throw new Error("mutation-test-record-missing");
  }
  return parsed;
};

const registryText = (invariant: InvariantRecord): string =>
  JSON.stringify({ invariants: [invariant], version: 1 });

const request = ({
  after,
  approval,
  before,
  files,
}: RequestInput): ApprovedRequest => {
  const registryFile = {
    path: registryPath,
    preimage: registryText(before),
    replacement: registryText(after),
  };
  const allFiles = [...files, registryFile];
  return {
    approval: {
      ...approval,
      manifest: {
        files: allFiles,
        registryDelta: {
          after,
          before,
          targetInvariantId: after.id,
        },
      },
    },
    preparedFiles: allFiles.map(({ path, preimage, replacement }) => ({
      contents: replacement,
      path,
      preimage,
    })),
    targetInvariantId: after.id,
  };
};

const promotionRecords = (
  sources: InvariantRecord["sources"] = [
    source("206"),
    source(secondPullRequest),
  ],
  oracle?: NonNullable<InvariantRecord["oracle"]>,
): RecordPair => {
  const before = record(candidate({ id: "validate-boundary-input", sources }));
  const after = record({
    ...before,
    approval: promotionApproval,
    lifecycle: "active",
    marginalAblation,
    ...(oracle === undefined ? {} : { oracle }),
    verification:
      oracle === undefined
        ? verifiedVerification
        : {
            ...verifiedVerification,
            lastRun: {
              ...verifiedVerification.lastRun,
              oracle: {
                invocation: oracle.invocation,
                name: oracle.name,
                testPath: oracle.testPath,
              },
            },
          },
  });
  return { after, before };
};

const promotionRequest = (
  sources?: InvariantRecord["sources"],
  surface: RequestFile = defaultPromotionSurface,
  oracle?: NonNullable<InvariantRecord["oracle"]>,
): ApprovedRequest => {
  const records = promotionRecords(sources, oracle);
  return request({
    after: records.after,
    approval: promotionApproval,
    before: records.before,
    files: [surface],
  });
};

const retirementRecords = (): RetirementPair => {
  const active = promotionRecords().after;
  const retired = record({
    ...active,
    approval: retirementApproval,
    lifecycle: "retired",
    retirement: {
      reason: "The control moved to an enforceable boundary.",
      retiredAt: retirementApproval.approvedAt,
    },
  });
  return { active, retired };
};

const retirementRequest = (
  surface: RequestFile = defaultRetirementSurface,
): ApprovedRequest => {
  const records = retirementRecords();
  return request({
    after: records.retired,
    approval: retirementApproval,
    before: records.active,
    files: [surface],
  });
};

export {
  promotionApproval,
  promotionRecords,
  promotionRequest,
  registryPath,
  retirementApproval,
  retirementRecords,
  retirementRequest,
  surfacePath,
};
export { marginalAblation } from "./invariant-registry-test-support.ts";
