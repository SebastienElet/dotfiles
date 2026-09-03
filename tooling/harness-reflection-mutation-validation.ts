import {
  type HarnessMutationRequest,
  type MutationTransition,
  parseHarnessMutationRequest,
} from "./harness-reflection-mutation-workflow-types.ts";
import {
  type InvariantRegistry,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { resolveSupportedTarget } from "./harness-reflection-mutation-surfaces.ts";
import { validateApprovedFileRequest } from "./harness-reflection-mutation-authorization.ts";
import { validateTransition } from "./harness-reflection-mutation-transition.ts";

const registryPath = "harness/invariants/registry.json";
type ApprovedFile = NonNullable<
  HarnessMutationRequest["approval"]
>["manifest"]["files"][number];

const parseRegistryText = (source: string): InvariantRegistry => {
  try {
    return parseInvariantRegistry(JSON.parse(source));
  } catch {
    throw new Error("approved-registry-invalid");
  }
};

const validateChangedFiles = (input: HarnessMutationRequest): void => {
  for (const file of input.approval?.manifest.files ?? []) {
    if (file.preimage === file.replacement) {
      throw new Error("approved-file-no-op");
    }
  }
};

const registryMutation = (input: HarnessMutationRequest): ApprovedFile => {
  const matches = input.approval?.manifest.files.filter(
    ({ path }) => path === registryPath,
  );
  const [match] = matches ?? [];
  if (matches?.length !== 1 || match === undefined || match.preimage === null) {
    throw new Error("approved-registry-file-required");
  }
  return match;
};

const parseRequest = (rawInput: unknown): HarnessMutationRequest => {
  try {
    return parseHarnessMutationRequest(rawInput);
  } catch {
    throw new Error("mutation-request-invalid");
  }
};

const exactCandidateText = (transition: MutationTransition): string => {
  const text = transition.target.marginalAblation?.candidateTextExact;
  if (transition.target.controlKind !== "probabilistic" || text === undefined) {
    throw new Error("candidate-text-required-for-supported-surface");
  }
  return text;
};

const validateConditionalSkillChange = (
  surfaces: readonly ApprovedFile[],
  transition: MutationTransition,
  candidateText: string,
): void => {
  if (surfaces.length > 0) {
    throw new Error("conditional-skill-registry-only");
  }
  if (transition.target.statement !== candidateText) {
    throw new Error("conditional-skill-statement-mismatch");
  }
};

const validateFileSurfaceChange = (
  surfaces: readonly ApprovedFile[],
  transition: MutationTransition,
  candidateText: string,
): void => {
  const target = resolveSupportedTarget(transition.target);
  const targetSurfaces = surfaces.filter(({ path }) => path === target.path);
  const [surface] = targetSurfaces;
  if (targetSurfaces.length !== 1 || surface === undefined) {
    throw new Error("unsupported-control-surface");
  }
  if (surfaces.length !== 1) {
    throw new Error("approved-surface-required");
  }
  const preimageHasText = surface.preimage?.includes(candidateText) ?? false;
  const replacementHasText = surface.replacement.includes(candidateText);
  if (
    transition.kind === "promotion" &&
    (preimageHasText || !replacementHasText)
  ) {
    throw new Error("promotion-candidate-text-not-added");
  }
  if (
    transition.kind === "retirement" &&
    (!preimageHasText || replacementHasText)
  ) {
    throw new Error("retirement-candidate-text-not-removed");
  }
};

const validateSurfaceChange = (
  input: HarnessMutationRequest,
  transition: MutationTransition,
): void => {
  const surfaces =
    input.approval?.manifest.files.filter(
      ({ path }) => path !== registryPath,
    ) ?? [];
  if (transition.kind === "link") {
    if (surfaces.length > 0) {
      throw new Error("link-surface-forbidden");
    }
    return;
  }
  const candidateText = exactCandidateText(transition);
  if (transition.target.surface === "conditional-skill") {
    validateConditionalSkillChange(surfaces, transition, candidateText);
    return;
  }
  validateFileSurfaceChange(surfaces, transition, candidateText);
};

const validateApprovedHarnessMutation = (
  rawInput: unknown,
): MutationTransition => {
  const input = parseRequest(rawInput);
  if (input.approval === undefined) {
    throw new Error("approval-attestation-required");
  }
  validateApprovedFileRequest(input, input.approval);
  validateChangedFiles(input);
  const registry = registryMutation(input);
  const transition = validateTransition(
    input,
    {
      current: parseRegistryText(registry.preimage ?? ""),
      proposed: parseRegistryText(registry.replacement),
    },
    input.approval,
  );
  validateSurfaceChange(input, transition);
  return transition;
};

const validateAppliedHarnessMutation = (
  rawInput: unknown,
  currentFiles: Readonly<Record<string, string | undefined>>,
): MutationTransition => {
  const transition = validateApprovedHarnessMutation(rawInput);
  const input = parseHarnessMutationRequest(rawInput);
  for (const file of input.approval?.manifest.files ?? []) {
    const expected =
      file.path === registryPath ? file.preimage : file.replacement;
    if (currentFiles[file.path] !== (expected ?? undefined)) {
      throw new Error("approved-file-current-content-mismatch");
    }
  }
  return transition;
};

export {
  registryPath,
  validateAppliedHarnessMutation,
  validateApprovedHarnessMutation,
};
