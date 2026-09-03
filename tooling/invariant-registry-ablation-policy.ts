import type {
  InvariantRecord,
  RegistryDiagnostic,
} from "./invariant-registry-schema.ts";

type MarginalAblation = NonNullable<InvariantRecord["marginalAblation"]>;

const diagnostic = (
  code: string,
  path: string,
  message: string,
): RegistryDiagnostic => ({ code, path, message });

const sameValues = (
  left: readonly string[],
  right: readonly string[],
): boolean =>
  left.length === right.length &&
  left.every((value, index) => value === right[index]);

const hasControlledConstants = (ablation: MarginalAblation): boolean =>
  sameValues(ablation.with.scenarios, ablation.without.scenarios) &&
  sameValues(ablation.with.environments, ablation.without.environments) &&
  ablation.with.replicates === ablation.without.replicates;

const expectedOutcomes = (condition: MarginalAblation["with"]): number =>
  condition.scenarios.length *
  condition.environments.length *
  condition.replicates;

const hasCompleteOutcomes = (ablation: MarginalAblation): boolean =>
  ablation.with.outcomes.length === expectedOutcomes(ablation.with) &&
  ablation.without.outcomes.length === expectedOutcomes(ablation.without);

const hasObservableDelta = (ablation: MarginalAblation): boolean =>
  !sameValues(ablation.with.outcomes, ablation.without.outcomes);

const hasEffectiveActivation = (ablation: MarginalAblation): boolean => {
  const activation = ablation.conditionalSkillActivation;
  if (activation === undefined) {
    return false;
  }
  return (
    activation.with.total === expectedOutcomes(ablation.with) &&
    activation.without.total === expectedOutcomes(ablation.without) &&
    activation.with.activated > activation.without.activated
  );
};

const evidenceFindings = (
  path: string,
  ablation: MarginalAblation,
): readonly RegistryDiagnostic[] => [
  ...(hasControlledConstants(ablation)
    ? []
    : [
        diagnostic(
          "uncontrolled-marginal-ablation",
          path,
          "Ablation conditions must share scenarios, environments, and replicates.",
        ),
      ]),
  ...(hasCompleteOutcomes(ablation)
    ? []
    : [
        diagnostic(
          "incomplete-ablation-replicates",
          path,
          "Every scenario, environment, and replicate requires an outcome.",
        ),
      ]),
  ...(hasObservableDelta(ablation)
    ? []
    : [
        diagnostic(
          "missing-observable-delta",
          `${path}.observableDelta`,
          "Ablation outcomes must expose an observable with/without delta.",
        ),
      ]),
];

const promotionStateFindings = (
  record: InvariantRecord,
  path: string,
  ablation: MarginalAblation,
): readonly RegistryDiagnostic[] => [
  ...(record.verification.state === "verified"
    ? []
    : [
        diagnostic(
          "unverified-marginal-ablation",
          `${path}.verification`,
          "Active probabilistic controls require verified ablation evidence.",
        ),
      ]),
  ...(record.surface !== "conditional-skill" || hasEffectiveActivation(ablation)
    ? []
    : [
        diagnostic(
          "ineffective-activation-measurement",
          `${path}.conditionalSkillActivation`,
          "Conditional-skill activation must be measured and improve with the candidate text.",
        ),
      ]),
  ...(record.surface !== "conditional-skill" ||
  ablation.candidateTextExact === record.statement
    ? []
    : [
        diagnostic(
          "conditional-skill-statement-mismatch",
          `${path}.candidateTextExact`,
          "Conditional-skill candidate text must equal the registry statement.",
        ),
      ]),
];

const ablationFindings = (
  record: InvariantRecord,
  path: string,
  ablation: MarginalAblation,
): readonly RegistryDiagnostic[] => [
  ...evidenceFindings(path, ablation),
  ...promotionStateFindings(record, path, ablation),
];

const marginalAblationDiagnostics = (
  record: InvariantRecord,
  path: string,
): readonly RegistryDiagnostic[] => {
  if (record.lifecycle !== "active" || record.controlKind !== "probabilistic") {
    return [];
  }
  const ablation = record.marginalAblation;
  return ablation === undefined
    ? [
        diagnostic(
          "missing-marginal-ablation",
          `${path}.marginalAblation`,
          "Active probabilistic controls require controlled marginal ablation.",
        ),
      ]
    : ablationFindings(record, `${path}.marginalAblation`, ablation);
};

export { marginalAblationDiagnostics };
