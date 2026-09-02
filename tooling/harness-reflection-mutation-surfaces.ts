import type {
  InvariantRecord,
  RegistryDiagnostic,
} from "./invariant-registry-contract.ts";

type ConsumerName = keyof InvariantRecord["consumers"];
type SupportedMechanism<Name extends ConsumerName> = Extract<
  InvariantRecord["consumers"][Name],
  Readonly<{ state: "supported" }>
>["mechanism"];
type TargetConsumers = Readonly<{
  [Name in ConsumerName]?: SupportedMechanism<Name>;
}>;
type SupportedTarget = Readonly<{
  consumers: TargetConsumers;
  owner: "agent-instructions" | "skill-manager";
  path: string;
}>;

const supportedTargets: Readonly<
  Partial<Record<InvariantRecord["surface"], SupportedTarget>>
> = {
  "always-loaded-instruction": {
    consumers: {
      claude: "claude-global-instruction",
      codex: "codex-global-instruction",
    },
    owner: "agent-instructions",
    path: "harness/AGENTS.md",
  },
  "conditional-skill": {
    consumers: {
      claude: "claude-user-skill",
      codex: "codex-user-skill",
      cursor: "cursor-user-skill",
    },
    owner: "skill-manager",
    path: "harness/skills/harness-reflection/SKILL.md",
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
  const target = supportedTargets[record.surface];
  if (target === undefined) {
    return [];
  }
  return (["claude", "codex", "cursor"] as const).flatMap((name) =>
    consumerMatches(record, name, target.consumers[name])
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
  const target = supportedTargets[record.surface];
  if (target === undefined) {
    throw new Error("unsupported-control-surface");
  }
  for (const consumerName of ["claude", "codex", "cursor"] as const) {
    const expectedMechanism = target.consumers[consumerName];
    if (!consumerMatches(record, consumerName, expectedMechanism)) {
      throw new Error("mutation-consumer-surface-mismatch");
    }
  }
  return target;
};

export { consumerSurfaceDiagnostics, resolveSupportedTarget };
