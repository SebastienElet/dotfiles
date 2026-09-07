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
    "link": [
      "prepare-registry-only-proposal",
      "prepare-exact-registry-diff",
      "await-exact-manifest-approval"
    ],
    "propose": [
      "select-and-propose-control-surface",
      "prepare-route-specific-exact-manifest",
      "await-exact-manifest-approval"
    ]
  },
  "approvalBranches": {
    "absent": ["render-report-without-mutation"],
    "granted": ["execute-approved-change-order"]
  },
  "workflowRoutes": {
    "manifestValidation": {
      "module": "tooling/harness-reflection-mutation-validation.ts",
      "export": "validateAppliedHarnessMutation"
    },
    "registryValidation": {
      "command": "bun tooling/invariant-registry-cli.ts"
    }
  },
  "approvedChangeOrder": {
    "registryOnly": [
      "prepare-registry-only-proposal",
      "prepare-exact-registry-diff",
      "present-exact-manifest-for-contextual-human-approval",
      "validate-approved-manifest",
      "write-approved-registry-replacement-only",
      "run-registry-cli-and-declared-oracles",
      "render-report"
    ],
    "surfaceAndRegistry": [
      "select-and-propose-control-surface",
      "prepare-route-specific-exact-manifest",
      "present-exact-manifest-for-contextual-human-approval",
      "apply-surface-with-required-owner",
      "run-required-owner-doctor-and-contracts",
      "validate-approved-manifest-and-applied-surface",
      "write-approved-registry-replacement-only",
      "run-registry-cli-and-declared-oracles",
      "render-report"
    ]
  },
  "surfaceOwners": {
    "always-loaded-instruction": {
      "owner": "agent-instructions",
      "path": "harness/AGENTS.md",
      "verification": "agent-instructions-contracts"
    },
    "conditional-skill": {
      "owner": "skill-manager",
      "pathFromRecord": "targetSkillPath",
      "verification": "skill-manager-doctor-and-contracts"
    },
    "project-local-contract": {
      "owner": "agent-instructions",
      "path": "AGENTS.md",
      "verification": "agent-instructions-contracts"
    }
  },
  "externalControlRoutes": {
    "application": "owner-specific-exact-diff-and-contract-before-registry-recording",
    "genericManifestValidator": "not-applicable",
    "surfaces": {
      "hook": ["scripts", "enforcement-code"],
      "permission": ["enforcement-code"],
      "lint": ["scripts", "enforcement-code"],
      "type": ["enforcement-code"],
      "architectural-test": ["enforcement-code"]
    }
  },
  "manifestValidation": {
    "appliesTo": [
      "always-loaded-instruction",
      "conditional-skill",
      "project-local-contract"
    ],
    "behavior": "read-only-no-file-writes",
    "candidateTextRule": "file-surfaces-add-or-remove-exact-text",
    "fileSurfaces": [
      "always-loaded-instruction",
      "conditional-skill",
      "project-local-contract"
    ],
    "conditionalSkillTarget": "existing-triggerable-user-skill-deployed-to-declared-consumers",
    "noOpRule": "every-approved-replacement-differs-from-preimage",
    "registrySurfaces": [],
    "semanticClaim": "exact-file-text-plus-owner-doctor-only",
    "transitionKind": "derived-from-before-and-after"
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
    "historicalFixtureCoverage": "dedup-policy-proposal-and-manifest-validation",
    "syntheticFixtures": "local-only-explicitly-not-historical-evidence"
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
    "selectionRequiredBeforeApproval": true
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
        "pathFromRecord": "targetSkillPath",
        "targetRequirement": "existing-triggerable-user-skill-deployed-to-declared-consumers",
        "consumers": {
          "claude": "claude-user-skill",
          "codex": "codex-user-skill",
          "cursor": "cursor-user-skill"
        }
      },
      "projectLocalContract": {
        "surface": "project-local-contract",
        "path": "AGENTS.md",
        "consumers": {
          "claude": "unsupported",
          "codex": "unsupported",
          "cursor": "unsupported"
        }
      }
    }
  },
  "oracle": {
    "requiredBeforeApproval": true,
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
    "durableValidityClaim": false,
    "libraryValidation": "structural-and-repository-policy-without-oracle-execution",
    "executableValidation": "runs-declared-oracles-for-verified-records-that-declare-one"
  },
  "retirement": {
    "approval": "new-exact-attestation-recorded",
    "historicalFields": "unchanged-except-approval-lifecycle-and-retirement",
    "optionalFields": ["replacedBy"],
    "requiredFields": ["retiredAt", "reason"],
    "surfaceText": "file-surface-text-removed"
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
