import { join } from "node:path";
import { z } from "zod";

const measurementSchema = z
  .object({
    outcome: z.enum(["passed", "failed"]),
    ranAt: z.iso.datetime(),
    environment: z.string().min(1),
  })
  .strict();

const verificationSchema = z.discriminatedUnion("state", [
  z.object({ state: z.literal("unverified") }).strict(),
  z
    .object({ state: z.literal("measured"), lastRun: measurementSchema })
    .strict(),
  z
    .object({
      state: z.literal("verified"),
      lastRun: measurementSchema.extend({ outcome: z.literal("passed") }),
    })
    .strict(),
]);

const consumerSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("supported"),
      mechanism: z.string().min(1),
      lastVerifiedEnvironment: z.string().min(1).optional(),
    })
    .strict(),
  z
    .object({ state: z.literal("unsupported"), reason: z.string().min(1) })
    .strict(),
]);

const invariantRecordSchema = z
  .object({
    id: z.string().min(1),
    statement: z.string().min(1),
    lifecycle: z.enum(["candidate", "active", "retired"]),
    controlKind: z.enum(["probabilistic", "enforceable"]),
    causeClass: z.enum([
      "not-applied",
      "not-loaded",
      "unknown",
      "blind-spot",
      "judgment",
    ]),
    severity: z.enum(["low", "medium", "high", "critical"]),
    sources: z
      .array(
        z
          .object({
            pullRequestUrl: z.url(),
            evidenceUrl: z.url(),
          })
          .strict(),
      )
      .min(1),
    scope: z
      .object({
        kind: z.enum(["cross-project", "project-local"]),
        exceptions: z.array(
          z
            .object({
              paths: z.array(z.string().min(1)).min(1),
              reason: z.string().min(1),
            })
            .strict(),
        ),
      })
      .strict(),
    surface: z.enum([
      "always-loaded-instruction",
      "conditional-skill",
      "project-local-contract",
      "hook",
      "permission",
      "lint",
      "type",
      "architectural-test",
    ]),
    approval: z
      .object({ approvedBy: z.string().min(1), approvedAt: z.iso.datetime() })
      .strict()
      .optional(),
    oracle: z
      .object({
        name: z.string().min(1),
        failurePath: z.string().min(1),
        testPath: z.string().min(1),
      })
      .strict()
      .optional(),
    consumers: z
      .object({
        claude: consumerSchema,
        codex: consumerSchema,
        cursor: consumerSchema,
      })
      .strict(),
    verification: verificationSchema,
    retirement: z
      .object({
        retiredAt: z.iso.datetime(),
        reason: z.string().min(1),
        replacedBy: z.string().min(1).optional(),
      })
      .strict()
      .optional(),
  })
  .strict();

const invariantRegistrySchema = z
  .object({ version: z.literal(1), invariants: z.array(invariantRecordSchema) })
  .strict();

type InvariantRecord = Readonly<z.output<typeof invariantRecordSchema>>;
type InvariantRegistry = Readonly<z.output<typeof invariantRegistrySchema>>;
type RegistryDiagnostic = Readonly<{
  code: string;
  path: string;
  message: string;
}>;
type ValidationOptions = Readonly<{
  repositoryRoot: string;
  pathExists: (path: string) => boolean;
}>;

const promotionThreshold = 2;

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({ code, path, message });

const surfaceMatchesControlKind = (record: InvariantRecord): boolean => {
  const probabilisticSurfaces = new Set([
    "always-loaded-instruction",
    "conditional-skill",
    "project-local-contract",
  ]);

  return record.controlKind === "probabilistic"
    ? probabilisticSurfaces.has(record.surface)
    : !probabilisticSurfaces.has(record.surface);
};

const promotionDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] => {
  if (record.lifecycle !== "active") {
    return [];
  }

  const pullRequestCount = new Set(
    record.sources.map(({ pullRequestUrl }) => pullRequestUrl),
  ).size;
  const diagnostics: RegistryDiagnostic[] = [];
  if (record.approval === undefined) {
    diagnostics.push(
      diagnostic(
        "missing-approval",
        `${path}.approval`,
        "Active invariants require explicit approval.",
      ),
    );
  }
  if (record.causeClass === "judgment") {
    diagnostics.push(
      diagnostic(
        "judgment-promotion",
        `${path}.causeClass`,
        "Judgment cannot be promoted to a control.",
      ),
    );
  }
  if (
    pullRequestCount < promotionThreshold &&
    !["high", "critical"].includes(record.severity)
  ) {
    diagnostics.push(
      diagnostic(
        "insufficient-promotion-evidence",
        `${path}.sources`,
        "Active invariants require two pull requests or high severity.",
      ),
    );
  }
  return diagnostics;
};

const recordDiagnostics = (
  record: InvariantRecord,
  index: number,
  invariantIds: ReadonlySet<string>,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const path = `invariants.${index}`;
  const diagnostics = [...promotionDiagnostics(record, path)];
  if (
    record.lifecycle === "candidate" &&
    record.verification.state !== "unverified"
  ) {
    diagnostics.push(
      diagnostic(
        "candidate-measured",
        `${path}.verification`,
        "Candidates must remain unverified.",
      ),
    );
  }
  if (!surfaceMatchesControlKind(record)) {
    diagnostics.push(
      diagnostic(
        "incompatible-surface",
        `${path}.surface`,
        "The surface is incompatible with the control kind.",
      ),
    );
  }
  if (record.lifecycle === "retired" && record.retirement === undefined) {
    diagnostics.push(
      diagnostic(
        "missing-retirement",
        `${path}.retirement`,
        "Retired invariants require a date and reason.",
      ),
    );
  }
  if (
    record.retirement?.replacedBy !== undefined &&
    !invariantIds.has(record.retirement.replacedBy)
  ) {
    diagnostics.push(
      diagnostic(
        "unknown-replacement",
        `${path}.retirement.replacedBy`,
        "Replacement invariant does not exist.",
      ),
    );
  }
  return [...diagnostics, ...oracleDiagnostics(record, path, options)];
};

const oracleDiagnostics = (
  record: InvariantRecord,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const requiresOracle =
    record.controlKind === "enforceable" &&
    (record.lifecycle === "active" || record.verification.state === "verified");
  if (!requiresOracle) {
    return [];
  }
  if (record.oracle === undefined) {
    return [
      diagnostic(
        "missing-oracle",
        `${path}.oracle`,
        "Enforceable active or verified invariants require an oracle.",
      ),
    ];
  }
  return options.pathExists(
    join(options.repositoryRoot, record.oracle.testPath),
  )
    ? []
    : [
        diagnostic(
          "missing-oracle-path",
          `${path}.oracle.testPath`,
          "Oracle test path does not exist.",
        ),
      ];
};

const parseInvariantRegistry = (input: unknown): InvariantRegistry =>
  invariantRegistrySchema.parse(input);

const validateInvariantRegistry = (
  registry: InvariantRegistry,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const invariantIds = new Set(registry.invariants.map(({ id }) => id));
  const sourcePaths = new Map<string, string>();
  const uniquenessDiagnostics = registry.invariants.flatMap((record, index) => {
    const path = `invariants.${index}`;
    const duplicateId =
      registry.invariants.findIndex(({ id }) => id === record.id) !== index;
    const sourceDiagnostics = record.sources.flatMap((source, sourceIndex) => {
      const sourcePath = `${path}.sources.${sourceIndex}.evidenceUrl`;
      const duplicate = sourcePaths.has(source.evidenceUrl);
      sourcePaths.set(source.evidenceUrl, sourcePath);
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
  return [
    ...uniquenessDiagnostics,
    ...registry.invariants.flatMap((record, index) =>
      recordDiagnostics(record, index, invariantIds, options),
    ),
  ];
};

export {
  parseInvariantRegistry,
  validateInvariantRegistry,
  type InvariantRecord,
  type InvariantRegistry,
  type RegistryDiagnostic,
  type ValidationOptions,
};
