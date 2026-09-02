# Invariant Registry

Use this reference only after the diagnostic class is `harness-gap`. The canonical registry remains
the source of truth for named invariant records. The JSON block below is the sole authority for the
reflection workflow; execute its `workflowOrder` literally and reject prose or data that contradicts
any closed value.

## Authoritative workflow contract

```json
{
  "version": 1,
  "workflowOrder": [
    "preserve-factual-evidence",
    "search-registry",
    "classify-registry-cause",
    "choose-decision"
  ],
  "decisionBranches": {
    "skip": ["render-report"],
    "link": ["hold-session-local", "render-report"],
    "propose": ["hold-session-local", "render-report"]
  },
  "approvedMutationOrder": [
    "require-approval",
    "select-control-surface",
    "declare-consumers",
    "require-oracle",
    "run-cli",
    "render-report"
  ],
  "registry": {
    "path": "harness/invariants/registry.json",
    "classes": ["not-applied", "not-loaded", "unknown", "blind-spot", "judgment"],
    "decisions": ["skip", "link", "propose"],
    "judgmentDecision": "skip",
    "existingInvariantDecision": "link",
    "linkEffect": "add-source-without-duplicate-record",
    "missingInvariantDecision": "propose-if-evidence-threshold-met"
  },
  "evidence": {
    "factualPrFeedback": "immutable",
    "concretePrUrls": "required",
    "missingEvidenceDecision": "skip",
    "promotionThreshold": "two-distinct-pull-requests-or-high-severity",
    "syntheticSources": "forbidden"
  },
  "approval": {
    "requiredBeforeMutation": true,
    "preApprovalState": "session-local",
    "timePressureBypass": false
  },
  "controls": {
    "probabilistic": ["always-loaded-instruction", "conditional-skill", "project-local-contract"],
    "enforceable": ["hook", "permission", "lint", "type", "architectural-test"],
    "selectionRequiredAfterApproval": true
  },
  "consumers": {
    "required": ["claude", "codex", "cursor"],
    "declaration": "independent-supported-mechanism-or-unsupported-reason"
  },
  "oracle": {
    "requiredAfterApproval": true,
    "enforceable": "executable-failure-path-and-test-path",
    "probabilistic": "behavioral-trial-with-environment",
    "inapplicable": "reason-required"
  },
  "routes": {
    "skillChange": "skill-manager",
    "instructionChange": "agent-instructions"
  },
  "cli": {
    "command": "bun tooling/invariant-registry-cli.ts",
    "timing": "immediately-before-report",
    "claim": "accepted-snapshot-read-in-execution-environment",
    "durableValidityClaim": false
  },
  "retirement": {
    "requiredFields": ["retiredAt", "reason"],
    "optionalFields": ["replacedBy"]
  },
  "proposal": {
    "requiredFields": [
      "trigger",
      "desired-behavior",
      "scope",
      "supporting-evidence",
      "counterexample",
      "falsifier",
      "expiry-condition",
      "cheapest-behavioral-trial"
    ]
  },
  "lifecycle": {
    "promotion": "three-independent-sessions-without-contradictory-result",
    "rollback": ["two-failed-trials", "one-safety-regression", "user-veto"]
  },
  "report": {
    "appliesToDecisions": ["skip", "link", "propose"],
    "requiredFields": [
      "registry-lookup",
      "decision-and-reason",
      "control-kind-and-surface",
      "sources-evidence-and-missing-proof",
      "oracle-or-inapplicability",
      "approval-status",
      "claude-consumer",
      "codex-consumer",
      "cursor-consumer",
      "cli-snapshot-result-or-not-run-reason"
    ]
  }
}
```
