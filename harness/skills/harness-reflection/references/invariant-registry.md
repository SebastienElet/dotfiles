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
    "mutation": {
      "module": "tooling/harness-reflection-mutation-workflow.ts",
      "export": "executeHarnessMutationWorkflow"
    }
  },
  "mutationExecution": {
    "guarantee": "cooperative-adapter-lock-with-best-effort-multi-file-compensation-not-atomic",
    "concurrencyScope": "mutations-through-owned-adapter-only",
    "nonCooperativeWriters": "outside-guarantee",
    "interruptionLimit": "hard-interruption-may-leave-lock-temp-or-partial-multi-file-change-without-output",
    "crashRecovery": "inspect-lock-temp-and-git-before-manual-cleanup-and-retry",
    "applyOrder": [
      "stage-all-replacements-in-same-directories",
      "revalidate-all-current-files-under-cooperative-lock",
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
      "select-supported-control-surface-and-exact-path",
      "declare-supported-consumer-mechanisms",
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
      "validate-persisted-approval-matches-accepted-attestation"
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
      "exact-paths",
      "exact-preimages",
      "exact-replacements",
      "target-invariant-id",
      "exact-target-before-and-after"
    ],
    "transitionKind": "derived-from-exact-target-before-and-after",
    "proceduralPrecondition": "contextual-human-approval-before-attestation",
    "codeAcceptance": "exact-approval-attestation-without-origin-authentication",
    "authentication": "not-performed",
    "registryRecordMeaning": "recorded-attestation-not-independent-proof",
    "agentSelfAssertion": "procedurally-forbidden-not-machine-detectable",
    "timePressureBypass": false
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
    "declaration": "independent-supported-mechanism-or-unsupported-reason",
    "supportedMechanisms": {
      "claude": ["claude-global-instruction", "claude-user-skill"],
      "codex": ["codex-global-instruction", "codex-user-skill"],
      "cursor": ["cursor-user-skill"]
    },
    "mutationTargets": {
      "alwaysLoadedInstruction": {
        "surface": "always-loaded-instruction",
        "path": "harness/AGENTS.md",
        "consumers": {
          "claude": "claude-global-instruction",
          "codex": "codex-global-instruction",
          "cursor": "unsupported"
        }
      },
      "conditionalSkill": {
        "surface": "conditional-skill",
        "path": "harness/skills/harness-reflection/SKILL.md",
        "consumers": {
          "claude": "claude-user-skill",
          "codex": "codex-user-skill",
          "cursor": "cursor-user-skill"
        }
      }
    }
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
      "preserve-historical-fields-in-prepared-registry",
      "set-retired-at-in-prepared-registry",
      "set-retirement-reason-in-prepared-registry",
      "handle-optional-replaced-by-in-prepared-registry",
      "record-new-approval-attestation-in-prepared-registry",
      "prepare-selected-control-surface-copy-if-touched",
      "capture-all-file-preimages-for-approval",
      "construct-exact-retirement-manifest",
      "await-human-context-approval-for-exact-manifest"
    ],
    "validationOrder": [
      "validate-request-equals-approved-manifest",
      "acquire-owned-cooperative-lock",
      "revalidate-approved-preimages-under-lock",
      "validate-historical-fields-unchanged-except-approval-lifecycle-and-retirement",
      "validate-prepared-selected-control-surface-if-touched-with-owned-adapter",
      "validate-prepared-retired-registry-with-owned-schema-and-policy",
      "validate-only-approved-target-registry-delta",
      "validate-persisted-approval-matches-accepted-attestation"
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
    "allowedTransitions": [
      "new-to-candidate",
      "new-to-active",
      "candidate-to-candidate",
      "candidate-to-active",
      "active-to-active",
      "active-to-retired"
    ],
    "retiredTerminal": true,
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
