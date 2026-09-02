import type { InvariantRecord } from "./invariant-registry-contract.ts";

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
    path: "harness/AGENTS.md",
  },
  "conditional-skill": {
    consumers: {
      claude: "claude-user-skill",
      codex: "codex-user-skill",
      cursor: "cursor-user-skill",
    },
    path: "harness/skills/harness-reflection/SKILL.md",
  },
};

const resolveSupportedTarget = (record: InvariantRecord): SupportedTarget => {
  const target = supportedTargets[record.surface];
  if (target === undefined) {
    throw new Error("unsupported-control-surface");
  }
  for (const consumerName of ["claude", "codex", "cursor"] as const) {
    const expectedMechanism = target.consumers[consumerName];
    const consumer = record.consumers[consumerName];
    if (
      (expectedMechanism === undefined && consumer.state !== "unsupported") ||
      (expectedMechanism !== undefined &&
        (consumer.state !== "supported" ||
          consumer.mechanism !== expectedMechanism))
    ) {
      throw new Error("mutation-consumer-surface-mismatch");
    }
  }
  return target;
};

export { resolveSupportedTarget };
