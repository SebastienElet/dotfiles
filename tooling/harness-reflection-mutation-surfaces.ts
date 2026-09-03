import type {
  InvariantRecord,
  RegistryDiagnostic,
} from "./invariant-registry-schema.ts";

type ConsumerName = keyof InvariantRecord["consumers"];
type SupportedMechanism<Name extends ConsumerName> = Extract<
  InvariantRecord["consumers"][Name],
  Readonly<{ state: "supported" }>
>["mechanism"];
type TargetConsumers = Readonly<{
  [Name in ConsumerName]?: SupportedMechanism<Name>;
}>;
type SurfaceOwner =
  | "agent-instructions"
  | "enforcement-code"
  | "scripts"
  | "skill-manager";
type SupportedTarget = Readonly<{
  consumers: TargetConsumers;
  owner: Extract<SurfaceOwner, "agent-instructions" | "skill-manager">;
  path: string;
  evaluationResetPath?: string;
}>;
type SurfaceRoute = Readonly<{
  consumers: TargetConsumers;
  owners: readonly [SurfaceOwner, ...SurfaceOwner[]];
  target?: SupportedTarget;
}>;

const noAgentConsumers = {};
const surfaceRoutes: Readonly<
  Record<InvariantRecord["surface"], SurfaceRoute>
> = {
  "always-loaded-instruction": {
    consumers: {
      claude: "claude-global-instruction",
      codex: "codex-global-instruction",
    },
    owners: ["agent-instructions"],
    target: {
      consumers: {
        claude: "claude-global-instruction",
        codex: "codex-global-instruction",
      },
      owner: "agent-instructions",
      path: "harness/AGENTS.md",
    },
  },
  "conditional-skill": {
    consumers: {
      claude: "claude-user-skill",
      codex: "codex-user-skill",
      cursor: "cursor-user-skill",
    },
    owners: ["skill-manager"],
    target: {
      consumers: {
        claude: "claude-user-skill",
        codex: "codex-user-skill",
        cursor: "cursor-user-skill",
      },
      evaluationResetPath:
        "harness/skills/harness-reflection/evals/promotion-workflow-results.json",
      owner: "skill-manager",
      path: "harness/skills/harness-reflection/SKILL.md",
    },
  },
  "project-local-contract": {
    consumers: noAgentConsumers,
    owners: ["agent-instructions"],
  },
  hook: {
    consumers: noAgentConsumers,
    owners: ["scripts", "enforcement-code"],
  },
  permission: {
    consumers: noAgentConsumers,
    owners: ["enforcement-code"],
  },
  lint: {
    consumers: noAgentConsumers,
    owners: ["scripts", "enforcement-code"],
  },
  type: {
    consumers: noAgentConsumers,
    owners: ["enforcement-code"],
  },
  "architectural-test": {
    consumers: noAgentConsumers,
    owners: ["enforcement-code"],
  },
};

const consumerMatches = (
  record: InvariantRecord,
  name: ConsumerName,
  expected: string | undefined,
): boolean => {
  const consumer = record.consumers[name];
  return expected === undefined
    ? consumer.state === "unsupported"
    : consumer.state === "supported" && consumer.mechanism === expected;
};

const consumerSurfaceDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] => {
  const route = surfaceRoutes[record.surface];
  return (["claude", "codex", "cursor"] as const).flatMap((name) =>
    consumerMatches(record, name, route.consumers[name])
      ? []
      : [
          {
            code: "consumer-surface-mismatch",
            message:
              "Consumer support must match the canonical surface projection.",
            path: `${path}.consumers.${name}`,
          },
        ],
  );
};

const resolveSupportedTarget = (record: InvariantRecord): SupportedTarget => {
  const { target } = surfaceRoutes[record.surface];
  if (target === undefined) {
    throw new Error("unsupported-control-surface");
  }
  for (const consumerName of ["claude", "codex", "cursor"] as const) {
    if (
      !consumerMatches(record, consumerName, target.consumers[consumerName])
    ) {
      throw new Error("mutation-consumer-surface-mismatch");
    }
  }
  return target;
};

export { consumerSurfaceDiagnostics, resolveSupportedTarget, surfaceRoutes };
