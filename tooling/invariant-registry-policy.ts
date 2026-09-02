import type {
  InvariantRecord,
  InvariantRegistry,
  RegistryDiagnostic,
  ValidationOptions,
} from "./invariant-registry-schema.ts";
import { oracleDiagnostics } from "./invariant-registry-oracle-policy.ts";
import { pullRequestIdentity } from "./invariant-registry-source.ts";

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
  const pullRequestIdentities = record.sources.map(({ pullRequestUrl }) =>
    pullRequestIdentity(pullRequestUrl),
  );
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
  ...(record.lifecycle === "retired" && record.retirement === undefined
    ? [
        diagnostic(
          "missing-retirement",
          `${path}.retirement`,
          "Retired invariants require a date and reason.",
        ),
      ]
    : []),
  ...(record.retirement?.replacedBy !== undefined &&
  !ids.has(record.retirement.replacedBy)
    ? [
        diagnostic(
          "unknown-replacement",
          `${path}.retirement.replacedBy`,
          "Replacement invariant does not exist.",
        ),
      ]
    : []),
  ...(record.retirement?.replacedBy === record.id
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
  const sourcePaths = new Map<string, string>();
  return registry.invariants.flatMap((record, index) => {
    const path = `invariants.${index}`;
    const duplicateId =
      registry.invariants.findIndex(({ id }) => id === record.id) !== index;
    const sourceDiagnostics = record.sources.flatMap((source, sourceIndex) => {
      const sourcePath = `${path}.sources.${sourceIndex}.pullRequestUrl`;
      const identity = pullRequestIdentity(source.pullRequestUrl);
      const duplicate = sourcePaths.has(identity);
      sourcePaths.set(identity, sourcePath);
      return duplicate
        ? [
            diagnostic(
              "duplicate-source",
              sourcePath,
              "Review source is already assigned to an invariant.",
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
      ...lifecycleDiagnostics(record, path, ids),
      ...oracleDiagnostics(record, path, options),
    ];
  });
  return [...uniquenessDiagnostics(registry), ...recordDiagnostics];
};

export { validateInvariantRegistry };
