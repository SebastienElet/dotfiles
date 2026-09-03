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
  | "skill-manager"
  | "scripts";
type SupportedTarget = Readonly<{
  consumers: TargetConsumers;
  owner: Extract<SurfaceOwner, "agent-instructions" | "skill-manager">;
  path: string;
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
  },
  "project-local-contract": {
    consumers: noAgentConsumers,
    owners: ["agent-instructions"],
    target: {
      consumers: noAgentConsumers,
      owner: "agent-instructions",
      path: "AGENTS.md",
    },
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
  const route = surfaceRoutes[record.surface];
  const target =
    record.surface === "conditional-skill"
      ? {
          consumers: route.consumers,
          owner: "skill-manager" as const,
          path: record.targetSkillPath,
        }
      : route.target;
  if (target === undefined) {
    throw new Error("unsupported-control-surface");
  }
  if (target.path === "harness/skills/harness-reflection/SKILL.md") {
    throw new Error("conditional-skill-self-target");
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
