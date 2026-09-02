import {
  type InvariantRegistry,
  type RegistryDiagnostic,
  type ValidationOptions,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";

type TestInvariant = Readonly<Record<string, unknown>>;
type ReviewSource = Readonly<{
  pullRequestUrl: string;
  evidenceUrl: string;
}>;

const firstPullRequest = "206";
const secondPullRequest = "207";
const verifiedVerification = {
  state: "verified",
  lastRun: {
    outcome: "passed",
    ranAt: "2026-09-02T00:00:00.000Z",
    environment: "controlled marginal ablation on macOS and Linux",
  },
};
const marginalAblation = {
  protocol: "controlled-marginal-ablation",
  candidateTextExact: "Always validate external input before domain use.",
  with: {
    scenarios: ["invalid boundary value"],
    environments: ["macOS", "Linux"],
    replicates: 3,
    outcomes: ["pass", "pass", "pass", "pass", "pass", "pass"],
  },
  without: {
    scenarios: ["invalid boundary value"],
    environments: ["macOS", "Linux"],
    replicates: 3,
    outcomes: ["fail", "fail", "fail", "fail", "fail", "fail"],
  },
  observableDelta: "Invalid input rejection changed from 0/3 to 3/3.",
};

function source(pullRequestNumber: string): ReviewSource {
  return {
    pullRequestUrl: `https://github.com/SebastienElet/dotfiles/pull/${pullRequestNumber}`,
    evidenceUrl: `https://github.com/SebastienElet/dotfiles/pull/${pullRequestNumber}#review`,
  };
}

function candidate(overrides: TestInvariant = {}): TestInvariant {
  return {
    id: "prevent-secret-leaks",
    statement: "Rejected fetch URLs never expose credentials.",
    lifecycle: "candidate",
    controlKind: "probabilistic",
    causeClass: "unknown",
    severity: "medium",
    sources: [
      {
        pullRequestUrl: "https://github.com/SebastienElet/dotfiles/pull/206",
        evidenceUrl:
          "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552",
      },
    ],
    scope: { kind: "cross-project", exceptions: [] },
    surface: "always-loaded-instruction",
    consumers: {
      claude: { state: "supported", mechanism: "always-loaded-instruction" },
      codex: { state: "supported", mechanism: "always-loaded-instruction" },
      cursor: {
        state: "unsupported",
        reason: "No managed instruction surface.",
      },
    },
    verification: { state: "unverified" },
    ...overrides,
  };
}

function active(overrides: TestInvariant = {}): TestInvariant {
  return candidate({
    lifecycle: "active",
    controlKind: "enforceable",
    surface: "hook",
    approval: {
      approvedBy: "Sebastien",
      approvedAt: "2026-09-02T00:00:00.000Z",
    },
    oracle: {
      name: "fetch-url-redaction",
      failurePath: "Rejected URLs do not expose credentials.",
      testPath: "tooling/fetch-url-redaction.test.ts",
    },
    sources: [source(firstPullRequest), source(secondPullRequest)],
    ...overrides,
  });
}

function registry(...invariants: readonly TestInvariant[]): InvariantRegistry {
  return parseInvariantRegistry({ version: 1, invariants });
}

function validationOptions(
  pathExists: ValidationOptions["pathExists"] = (): boolean => true,
): ValidationOptions {
  return { repositoryRoot: "/repository", pathExists };
}

function diagnosticCodes(
  diagnostics: readonly RegistryDiagnostic[],
): readonly string[] {
  return diagnostics.map(({ code }) => code);
}

export {
  active,
  candidate,
  diagnosticCodes,
  firstPullRequest,
  marginalAblation,
  registry,
  secondPullRequest,
  source,
  type TestInvariant,
  validationOptions,
  verifiedVerification,
};
export { validateInvariantRegistry } from "./invariant-registry-contract.ts";
