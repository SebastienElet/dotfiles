const expectedCapabilities = [
  "durable_detection",
  "complete_proposal",
  "no_implicit_write",
  "authorized_admission",
  "stored",
  "fresh_retrieval",
  "proof_before_influence",
  "freshness_before_influence",
  "unrelated_not_injected",
  "sensitive_rejected",
  "rejection_redacted",
  "store_unchanged",
  "unavailable_omitted",
  "unavailable_no_mutation",
  "contradiction_invalidated",
] as const;

function capabilityChecks(
  evidence: Readonly<Record<string, unknown>>,
  nonce: string,
): Record<(typeof expectedCapabilities)[number], boolean> {
  const read = (name: string): string =>
    typeof evidence[name] === "string" ? evidence[name] : "";
  const runtimeObserved = evidence.runtimeObserved === true;
  const lifecycle = booleanRecord(evidence.lifecycle);
  const proposal = booleanRecord(evidence.proposalValidation);
  return {
    authorized_admission: evidence.admissionObserved === true,
    complete_proposal: proposal.stored === true,
    contradiction_invalidated: lifecycle.contradictionInvalidated === true,
    durable_detection: proposal.statementDetected === true,
    fresh_retrieval:
      runtimeObserved &&
      evidence.adapterValid === true &&
      evidence.storeArtifactsValid === true,
    freshness_before_influence:
      runtimeObserved && evidence.contextObserved === true,
    no_implicit_write:
      read("proposalState") === read("afterProposal") &&
      proposal.evaluatedStoreUnchanged === true,
    proof_before_influence:
      runtimeObserved && evidence.contextObserved === true,
    rejection_redacted: evidence.sensitiveRedacted === true,
    sensitive_rejected: evidence.sensitiveRefused === true,
    store_unchanged: evidence.sensitiveUnchanged === true,
    stored:
      evidence.admissionStored === true &&
      evidence.storeArtifactsValid === true,
    unavailable_no_mutation: lifecycle.unavailableNoMutation === true,
    unavailable_omitted: lifecycle.unavailableOmitted === true,
    unrelated_not_injected:
      evidence.controlUnchanged === true &&
      evidence.unrelatedUnchanged === true &&
      !read("controlText").includes(nonce) &&
      !read("unrelatedText").includes(nonce),
  };
}

function booleanRecord(value: unknown): Readonly<Record<string, boolean>> {
  if (!isRecord(value)) {
    return {};
  }
  const entries: [string, boolean][] = [];
  for (const [key, field] of Object.entries(value)) {
    if (typeof field === "boolean") {
      entries.push([key, field]);
    }
  }
  return Object.fromEntries(entries);
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { capabilityChecks, expectedCapabilities };
