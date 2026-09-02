# Invariant Registry

## Authoritative workflow contract

```json
{
  "version": 1,
  "initialWorkflowOrder": [
    "identify-equivalent-failure",
    "preserve-factual-evidence",
    "inspect-current-guidance",
    "classify-diagnostic-cause",
    "gate-registry-access"
  ],
  "diagnostic": {
    "classes": [
      "task-specific",
      "owned-defect",
      "external-transient",
      "missing-capability",
      "harness-gap"
    ],
    "harnessGap": "execute-harness-gap-workflow",
    "other": "skip-with-reason-and-next-diagnostic-action",
    "registryAccessForOther": "forbidden"
  },
  "harnessGapWorkflowOrder": [
    "read-authoritative-reference",
    "search-registry",
    "record-registry-lookup",
    "evaluate-concrete-evidence",
    "branch-on-evidence"
  ],
  "decisionBranches": {
    "skip": ["render-report"],
    "link": ["hold-session-local", "await-explicit-approval"],
    "propose": ["hold-session-local", "await-explicit-approval"]
  },
  "approvalBranches": {
    "absent": ["render-report-without-mutation"],
    "granted": ["execute-approved-compensated-mutation"]
  },
  "workflowRoutes": {
    "approvedMutation": {
      "module": "tooling/harness-reflection-mutation-workflow.ts",
      "export": "executeHarnessMutationWorkflow",
      "mode": "approved-mutation"
    },
    "retirement": {
      "module": "tooling/harness-reflection-mutation-workflow.ts",
      "export": "executeHarnessMutationWorkflow",
      "mode": "retirement"
    }
  },
  "mutationExecution": {
    "guarantee": "cooperative-adapter-lock-with-best-effort-multi-file-compensation-not-atomic",
    "concurrencyScope": "mutations-through-owned-adapter-only",
    "nonCooperativeWriters": "outside-guarantee",
    "interruptionLimit": "hard-interruption-may-leave-lock-temp-or-partial-multi-file-change-without-output",
    "crashRecovery": "inspect-lock-temp-and-git-before-manual-cleanup-and-retry",
    "applyOrder": [
      "stage-each-replacement-in-same-directory",
      "revalidate-current-file-under-cooperative-lock",
      "atomically-rename-each-file",
      "validate-applied-coherent-change"
    ],
    "onAnyError": [
      "reconcile-ambiguous-file-outcome",
      "compensate-applied-files-with-atomic-replacement-when-still-matching",
      "report-unresolved-files",
      "report-failure"
    ],
    "successOrder": ["render-report"]
  },
  "approvedMutation": {
    "execution": "mutationExecution",
    "prepareOrder": [
      "select-control-surface",
      "declare-consumers",
      "require-control-oracle",
      "prepare-selected-control-surface",
      "prepare-registry",
      "capture-all-file-preimages-for-approval",
      "construct-exact-mutation-manifest",
      "await-human-context-approval-for-exact-manifest"
    ],
    "validationOrder": [
      "validate-request-equals-approved-manifest",
      "acquire-owned-cooperative-lock",
      "revalidate-approved-preimages-under-lock",
      "validate-prepared-selected-control-surface-with-owned-adapter",
      "validate-prepared-registry-with-owned-schema-and-policy",
      "validate-only-approved-target-registry-delta",
      "validate-persisted-approval-matches-human-context"
    ]
  },
  "registry": {
    "path": "harness/invariants/registry.json",
    "classes": [
      "not-applied",
      "not-loaded",
      "unknown",
      "blind-spot",
      "judgment"
    ],
    "decisions": ["skip", "link", "propose"],
    "judgmentDecision": "skip",
    "existingInvariantDecision": "link",
    "linkEffect": "add-source-without-duplicate-record",
    "missingInvariantDecision": "propose-if-evidence-threshold-met"
  },
  "evidence": {
    "factualPrFeedback": "immutable",
    "concretePrUrls": "required",
    "evaluationTiming": "after-registry-lookup-recorded",
    "missingEvidenceDecision": "skip",
    "missingEvidenceWorkflow": ["choose-skip", "render-report"],
    "presentEvidenceWorkflow": ["classify-registry-cause", "choose-decision"],
    "promotionThreshold": "two-distinct-pull-requests-or-high-severity",
    "prFeedbackBoundary": {
      "input": "provided-factual-report-only",
      "directForgeIngestion": "forbidden",
      "historicalReconstruction": "forbidden",
      "collectionRole": "none"
    },
    "syntheticSources": "forbidden"
  },
  "approval": {
    "requiredBeforeMutation": true,
    "preApprovalState": "session-local",
    "manifestRequired": true,
    "manifestTiming": "present-exact-manifest-before-approval",
    "manifestContents": [
      "kind",
      "exact-paths",
      "exact-preimages",
      "exact-replacements",
      "target-invariant-id",
      "exact-target-before-and-after"
    ],
    "timePressureBypass": false,
    "inputSource": "human-context",
    "authentication": "not-performed",
    "registryRecordMeaning": "provided-context-not-independent-proof",
    "agentSelfAssertion": "forbidden"
  },
  "controls": {
    "probabilistic": [
      "always-loaded-instruction",
      "conditional-skill",
      "project-local-contract"
    ],
    "enforceable": ["hook", "permission", "lint", "type", "architectural-test"],
    "probabilisticPromotion": {
      "protocol": "controlled-marginal-ablation",
      "conditions": ["with-exact-candidate-text", "without-candidate-text"],
      "controlledConstants": ["scenarios", "environments", "replicates"],
      "observableDelta": "required",
      "withOnlyRuns": "never-sufficient",
      "activationMeasurementForConditionalSkill": "required"
    },
    "selectionRequiredAfterApproval": true
  },
  "consumers": {
    "required": ["claude", "codex", "cursor"],
    "declaration": "independent-supported-mechanism-or-unsupported-reason"
  },
  "oracle": {
    "requiredAfterApproval": true,
    "enforceable": "executable-failure-path-and-test-path",
    "probabilistic": "controlled-marginal-ablation-with-observable-delta",
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
    "execution": "mutationExecution",
    "requiredFields": ["retiredAt", "reason"],
    "optionalFields": ["replacedBy"],
    "prepareOrder": [
      "lookup-existing-invariant",
      "prepare-retired-registry-copy",
      "preserve-complete-record-history-in-prepared-registry",
      "set-retired-at-in-prepared-registry",
      "set-retirement-reason-in-prepared-registry",
      "handle-optional-replaced-by-in-prepared-registry",
      "prepare-selected-control-surface-copy-if-touched",
      "capture-all-file-preimages-for-approval",
      "construct-exact-retirement-manifest",
      "await-human-context-approval-for-exact-manifest"
    ],
    "validationOrder": [
      "validate-request-equals-approved-manifest",
      "acquire-owned-cooperative-lock",
      "revalidate-approved-preimages-under-lock",
      "validate-complete-record-history-unchanged",
      "validate-prepared-selected-control-surface-if-touched-with-owned-adapter",
      "validate-prepared-retired-registry-with-owned-schema-and-policy",
      "validate-only-approved-target-registry-delta",
      "validate-persisted-approval-matches-human-context"
    ]
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
    "promotion": "control-kind-specific-green-oracle-required",
    "independentWithOnlySessions": "never-sufficient-for-probabilistic-control",
    "rollback": ["two-failed-trials", "one-safety-regression", "user-veto"]
  },
  "report": {
    "appliesToDecisions": ["skip", "link", "propose"],
    "registryLookupAfterHarnessGap": "required-even-when-evidence-missing",
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
