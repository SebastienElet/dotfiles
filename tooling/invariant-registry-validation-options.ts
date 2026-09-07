type ConsumerName = "claude" | "codex" | "cursor";

type OracleInspection = Readonly<{
  discovered: boolean;
  kind: "missing" | "non-regular" | "regular-file";
  tracked: boolean;
}>;

type SkillTargetInspection = Readonly<{
  deploymentManifestValid: boolean;
  descriptionTriggerable: boolean;
  frontmatterValid: boolean;
  installedFor: readonly ConsumerName[];
  kind: "missing" | "non-regular" | "regular-file";
  name: string | undefined;
  tracked: boolean;
}>;

type ValidationOptions = Readonly<{
  repositoryRoot: string;
  inspectOracle: (
    path: string,
    invocation: readonly string[],
  ) => OracleInspection;
  inspectSkillTarget: (path: string) => SkillTargetInspection;
}>;

export type {
  ConsumerName,
  OracleInspection,
  SkillTargetInspection,
  ValidationOptions,
};
