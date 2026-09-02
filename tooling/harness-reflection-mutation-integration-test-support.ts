import {
  candidate,
  firstPullRequest,
  marginalAblation,
  secondPullRequest,
  source,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import {
  parseMutationManifest,
  parseMutationWorkflowCoreInput,
} from "./harness-reflection-mutation-workflow-types.ts";
import type { InvariantRecord } from "./invariant-registry-contract.ts";
import type { MutationWorkflowCoreInput } from "./harness-reflection-mutation-workflow-types.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";

const registryPath = "harness/invariants/registry.json";
const surfacePath = "harness/AGENTS.md";
const oraclePath = "tooling/invariant-registry-source-coherence.test.ts";
const targetInvariantId = "require-coherent-review-sources";

type ApprovalRecord = Readonly<{ approvedAt: string; approvedBy: string }>;
type PromotionFixture = Readonly<{
  activeRecord: InvariantRecord;
  activeRegistry: string;
  approval: ApprovalRecord;
  candidateRecord: InvariantRecord;
  candidateRegistry: string;
}>;
type RetirementFixture = Readonly<{
  approval: ApprovalRecord;
  record: InvariantRecord;
  registry: string;
}>;

const approvalRecord = (
  approvedAt: string,
  approvedBy: string,
): ApprovalRecord => ({ approvedAt, approvedBy });

const registryText = (record: InvariantRecord): string =>
  JSON.stringify({ invariants: [record], version: 1 });

const singleRecord = (value: unknown, label: string): InvariantRecord => {
  const [record] = parseInvariantRegistry(value).invariants;
  if (record === undefined) {
    throw new Error(`${label}-record-missing`);
  }
  return record;
};

const promotionOracle = (): NonNullable<InvariantRecord["oracle"]> => ({
  failurePath: "incoherent forge review sources are rejected",
  invocation: ["bun", "test", oraclePath],
  name: "invariant-registry-source-coherence",
  testPath: oraclePath,
});

const promotionFixture = (): PromotionFixture => {
  const candidateRecord = singleRecord(
    {
      invariants: [
        candidate({
          id: targetInvariantId,
          sources: [source(firstPullRequest), source(secondPullRequest)],
        }),
      ],
      version: 1,
    },
    "candidate",
  );
  const approval = approvalRecord("2026-09-02T09:00:00.000Z", "Reviewer");
  const oracle = promotionOracle();
  const activeRecord = singleRecord(
    {
      invariants: [
        {
          ...candidateRecord,
          approval,
          lifecycle: "active",
          marginalAblation,
          oracle,
          verification: {
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
        },
      ],
      version: 1,
    },
    "active",
  );
  return {
    activeRecord,
    activeRegistry: registryText(activeRecord),
    approval,
    candidateRecord,
    candidateRegistry: registryText(candidateRecord),
  };
};

const promotionRequest = (
  fixture: PromotionFixture,
): MutationWorkflowCoreInput =>
  parseMutationWorkflowCoreInput({
    approval: {
      ...fixture.approval,
      manifest: parseMutationManifest({
        files: [
          {
            path: surfacePath,
            preimage: "old guidance",
            replacement: "Validate review-source coherence.",
          },
          {
            path: registryPath,
            preimage: fixture.candidateRegistry,
            replacement: fixture.activeRegistry,
          },
        ],
        registryDelta: {
          after: fixture.activeRecord,
          before: fixture.candidateRecord,
          targetInvariantId,
        },
      }),
    },
    preparedFiles: [
      { contents: "Validate review-source coherence.", path: surfacePath },
      { contents: fixture.activeRegistry, path: registryPath },
    ],
    registryPath,
    targetInvariantId,
  });

const retirementFixture = (active: PromotionFixture): RetirementFixture => {
  const approval = approvalRecord(
    "2026-09-03T09:00:00.000Z",
    "Second reviewer",
  );
  const record = singleRecord(
    {
      invariants: [
        {
          ...active.activeRecord,
          approval,
          lifecycle: "retired",
          retirement: {
            reason: "The source boundary is now enforced elsewhere.",
            retiredAt: approval.approvedAt,
          },
        },
      ],
      version: 1,
    },
    "retired",
  );
  return { approval, record, registry: registryText(record) };
};

const retirementRequest = (
  active: PromotionFixture,
  retired: RetirementFixture,
): MutationWorkflowCoreInput =>
  parseMutationWorkflowCoreInput({
    approval: {
      ...retired.approval,
      manifest: parseMutationManifest({
        files: [
          {
            path: registryPath,
            preimage: active.activeRegistry,
            replacement: retired.registry,
          },
        ],
        registryDelta: {
          after: retired.record,
          before: active.activeRecord,
          targetInvariantId,
        },
      }),
    },
    preparedFiles: [{ contents: retired.registry, path: registryPath }],
    registryPath,
    targetInvariantId,
  });

export {
  oraclePath,
  promotionFixture,
  promotionRequest,
  registryPath,
  retirementFixture,
  retirementRequest,
  surfacePath,
};
export type { PromotionFixture, RetirementFixture };
