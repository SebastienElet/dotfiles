import type {
  InvariantRecord,
  InvariantRegistry,
  RegistryDiagnostic,
  ValidationOptions,
} from "./invariant-registry-schema.ts";
import {
  evidenceOccurrenceIdentity,
  pullRequestIdentity,
} from "./invariant-registry-source.ts";
import { conditionalSkillTargetDiagnostics } from "./invariant-registry-skill-target-policy.ts";
import { consumerSurfaceDiagnostics } from "./harness-reflection-mutation-surfaces.ts";
import { marginalAblationDiagnostics } from "./invariant-registry-ablation-policy.ts";
import { oracleDiagnostics } from "./invariant-registry-oracle-policy.ts";

type InvariantIds = Readonly<{ has: (id: string) => boolean }>;

const promotionThreshold = 2;
const probabilisticSurfaces = new Set([
  "always-loaded-instruction",
  "conditional-skill",
  "project-local-contract",
]);

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({
  code,
  path,
  message,
});

const promotionDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] => {
  if (record.lifecycle !== "active") {
    return [];
  }
  const pullRequestIdentities = record.sources.map(pullRequestIdentity);
  const enoughEvidence =
    new Set(pullRequestIdentities).size >= promotionThreshold ||
    ["high", "critical"].includes(record.severity);
  return [
    ...(record.approval === undefined
      ? [
          diagnostic(
            "missing-approval",
            `${path}.approval`,
            "Active invariants require explicit approval.",
          ),
        ]
      : []),
    ...(record.causeClass === "judgment"
      ? [
          diagnostic(
            "judgment-promotion",
            `${path}.causeClass`,
            "Judgment cannot be promoted to a control.",
          ),
        ]
      : []),
    ...(enoughEvidence
      ? []
      : [
          diagnostic(
            "insufficient-promotion-evidence",
            `${path}.sources`,
            "Active invariants require two pull requests or high severity.",
          ),
        ]),
  ];
};

const surfaceDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] =>
  probabilisticSurfaces.has(record.surface) ===
  (record.controlKind === "probabilistic")
    ? []
    : [
        diagnostic(
          "incompatible-surface",
          `${path}.surface`,
          "The surface is incompatible with the control kind.",
        ),
      ];

const lifecycleDiagnostics = (
  record: InvariantRecord,
  path: string,
  ids: InvariantIds,
): readonly RegistryDiagnostic[] => [
  ...(record.lifecycle === "candidate" &&
  record.verification.state !== "unverified"
    ? [
        diagnostic(
          "candidate-measured",
          `${path}.verification`,
          "Candidates must remain unverified.",
        ),
      ]
    : []),
  ...(record.lifecycle === "retired" &&
  record.retirement.replacedBy !== undefined &&
  !ids.has(record.retirement.replacedBy)
    ? [
        diagnostic(
          "unknown-replacement",
          `${path}.retirement.replacedBy`,
          "Replacement invariant does not exist.",
        ),
      ]
    : []),
  ...(record.lifecycle === "retired" &&
  record.retirement.replacedBy === record.id
    ? [
        diagnostic(
          "self-replacement",
          `${path}.retirement.replacedBy`,
          "Replacement invariant cannot be itself.",
        ),
      ]
    : []),
];

const uniquenessDiagnostics = (
  registry: InvariantRegistry,
): readonly RegistryDiagnostic[] => {
  const evidencePaths = new Map<string, string>();
  return registry.invariants.flatMap((record, index) => {
    const path = `invariants.${index}`;
    const duplicateId =
      registry.invariants.findIndex(({ id }) => id === record.id) !== index;
    const sourceDiagnostics = record.sources.flatMap((source, sourceIndex) => {
      const sourcePath = `${path}.sources.${sourceIndex}.evidenceUrl`;
      const identity = evidenceOccurrenceIdentity(source);
      const duplicate = evidencePaths.has(identity);
      evidencePaths.set(identity, sourcePath);
      return duplicate
        ? [
            diagnostic(
              "duplicate-evidence",
              sourcePath,
              "Review evidence is already assigned to an invariant.",
            ),
          ]
        : [];
    });
    return [
      ...(duplicateId
        ? [
            diagnostic(
              "duplicate-id",
              `${path}.id`,
              "Invariant identifier must be unique.",
            ),
          ]
        : []),
      ...sourceDiagnostics,
    ];
  });
};

const replacementCycleDiagnostics = (
  registry: InvariantRegistry,
): readonly RegistryDiagnostic[] => {
  const replacements = new Map(
    registry.invariants.flatMap((record) =>
      record.lifecycle === "retired" &&
      record.retirement.replacedBy !== undefined
        ? [[record.id, record.retirement.replacedBy] as const]
        : [],
    ),
  );
  return registry.invariants.flatMap((record, index) => {
    if (record.lifecycle !== "retired") {
      return [];
    }
    const visited = new Set([record.id]);
    let replacement = record.retirement.replacedBy;
    while (replacement !== undefined && replacements.has(replacement)) {
      if (visited.has(replacement)) {
        return [
          diagnostic(
            "replacement-cycle",
            `invariants.${index}.retirement.replacedBy`,
            "Replacement graph must be acyclic.",
          ),
        ];
      }
      visited.add(replacement);
      replacement = replacements.get(replacement);
    }
    return [];
  });
};

const validateInvariantRegistry = (
  registry: InvariantRegistry,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const ids = new Set(registry.invariants.map(({ id }) => id));
  const recordDiagnostics = registry.invariants.flatMap((record, index) => {
    const path = `invariants.${index}`;
    return [
      ...promotionDiagnostics(record, path),
      ...surfaceDiagnostics(record, path),
      ...consumerSurfaceDiagnostics(record, path),
      ...conditionalSkillTargetDiagnostics(record, path, options),
      ...marginalAblationDiagnostics(record, path),
      ...lifecycleDiagnostics(record, path, ids),
      ...oracleDiagnostics(record, path, options),
    ];
  });
  return [
    ...uniquenessDiagnostics(registry),
    ...replacementCycleDiagnostics(registry),
    ...recordDiagnostics,
  ];
};

export { validateInvariantRegistry };
