type ProposalValidation = Readonly<{
  evaluatedStoreUnchanged: boolean;
  statementDetected: boolean;
  stored: boolean;
}>;
type RecoveryRelation = Readonly<{
  profile: string;
  seed: string;
  window: string;
}>;

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function proposalStatesAcceptedRecovery(
  proposal: string,
  relation: RecoveryRelation,
): boolean {
  const parsed: unknown = Bun.YAML.parse(proposal);
  if (!isRecord(parsed) || typeof parsed.statement !== "string") {
    throw new Error("proposal statement is missing");
  }
  const statement = parsed.statement.toLowerCase();
  const mentionsRelation = [
    relation.window,
    relation.seed,
    relation.profile,
  ].every((term) => statement.includes(term.toLowerCase()));
  if (
    !mentionsRelation ||
    !/\baccepted?\b/u.test(statement) ||
    /\b(?:not|never)\s+(?:the\s+)?accepted?\b/u.test(statement)
  ) {
    throw new Error("proposal does not state the accepted recovery relation");
  }
  return true;
}

export {
  isRecord,
  proposalStatesAcceptedRecovery,
  type ProposalValidation,
  type RecoveryRelation,
};
