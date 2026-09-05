import type {
  InvariantRecord,
  RegistryDiagnostic,
  SkillTargetInspection,
  ValidationOptions,
} from "./invariant-registry-schema.ts";

const closedRouterPath = "harness/skills/harness-reflection/SKILL.md";
const consumerNames = ["claude", "codex", "cursor"] as const;

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({ code, message, path });

const targetName = (path: string): string | undefined => path.split("/")[2];

const targetDeploymentManifestDiagnostics = (
  inspection: SkillTargetInspection,
  targetPath: string,
): readonly RegistryDiagnostic[] =>
  inspection.deploymentManifestValid
    ? []
    : [
        diagnostic(
          "conditional-skill-deployment-invalid",
          targetPath,
          "Conditional skill deployment manifest is invalid.",
        ),
      ];

const targetMetadataDiagnostics = (
  inspection: SkillTargetInspection,
  targetPath: string,
  expectedName: string | undefined,
): readonly RegistryDiagnostic[] => [
  ...(inspection.tracked
    ? []
    : [
        diagnostic(
          "conditional-skill-target-untracked",
          targetPath,
          "Conditional skill target must be tracked by Git.",
        ),
      ]),
  ...(inspection.frontmatterValid
    ? []
    : [
        diagnostic(
          "conditional-skill-frontmatter-invalid",
          targetPath,
          "Conditional skill target frontmatter is invalid.",
        ),
      ]),
  ...(inspection.frontmatterValid && inspection.name !== expectedName
    ? [
        diagnostic(
          "conditional-skill-name-mismatch",
          targetPath,
          "Conditional skill name must equal its directory name.",
        ),
      ]
    : []),
  ...(inspection.frontmatterValid && !inspection.descriptionTriggerable
    ? [
        diagnostic(
          "conditional-skill-not-triggerable",
          targetPath,
          "Conditional skill description must support implicit discovery.",
        ),
      ]
    : []),
  ...targetDeploymentManifestDiagnostics(inspection, targetPath),
];

const consumerDeploymentDiagnostics = (
  record: Extract<InvariantRecord, { surface: "conditional-skill" }>,
  path: string,
  deployment: readonly string[],
): readonly RegistryDiagnostic[] =>
  consumerNames.flatMap((consumerName) => {
    const consumer = record.consumers[consumerName];
    return consumer.state === "supported" && !deployment.includes(consumerName)
      ? [
          diagnostic(
            "conditional-skill-consumer-not-deployed",
            `${path}.consumers.${consumerName}`,
            "Declared user-skill consumer has no matching user deployment.",
          ),
        ]
      : [];
  });

const deploymentDiagnostics = (
  record: Extract<InvariantRecord, { surface: "conditional-skill" }>,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  const inspection = options.inspectSkillTarget(record.targetSkillPath);
  const targetPath = `${path}.targetSkillPath`;
  if (inspection.kind === "missing") {
    return [
      diagnostic(
        "conditional-skill-target-missing",
        targetPath,
        "Conditional skill target does not exist.",
      ),
    ];
  }
  if (inspection.kind !== "regular-file") {
    return [
      diagnostic(
        "conditional-skill-target-not-regular",
        targetPath,
        "Conditional skill target must be a regular file.",
      ),
    ];
  }
  const expectedName = targetName(record.targetSkillPath);
  return [
    ...targetMetadataDiagnostics(inspection, targetPath, expectedName),
    ...consumerDeploymentDiagnostics(record, path, inspection.installedFor),
  ];
};

const conditionalSkillTargetDiagnostics = (
  record: InvariantRecord,
  path: string,
  options: ValidationOptions,
): readonly RegistryDiagnostic[] => {
  if (record.surface !== "conditional-skill") {
    return [];
  }
  if (record.targetSkillPath === closedRouterPath) {
    return [
      diagnostic(
        "conditional-skill-self-target",
        `${path}.targetSkillPath`,
        "The closed harness-reflection router cannot be a conditional target.",
      ),
    ];
  }
  return deploymentDiagnostics(record, path, options);
};

export { conditionalSkillTargetDiagnostics };
