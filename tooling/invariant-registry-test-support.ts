import {
  type InvariantRegistry,
  type OracleInspection,
  type RegistryDiagnostic,
  type ValidationOptions,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";

type TestInvariant = Readonly<Record<string, unknown>>;
type ReviewSource = Readonly<{
  provider: "github";
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
const oracle = {
  name: "fetch-url-redaction",
  failurePath: "Rejected URLs do not expose credentials.",
  testPath: "tooling/fetch-url-redaction.test.ts",
  invocation: ["bun", "test", "tooling/fetch-url-redaction.test.ts"],
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
    provider: "github",
    pullRequestUrl: `https://github.com/SebastienElet/dotfiles/pull/${pullRequestNumber}`,
    evidenceUrl: `https://github.com/SebastienElet/dotfiles/pull/${pullRequestNumber}#issuecomment-${pullRequestNumber}`,
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
        provider: "github",
        pullRequestUrl: "https://github.com/SebastienElet/dotfiles/pull/206",
        evidenceUrl:
          "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552",
      },
    ],
    scope: { kind: "cross-project", exceptions: [] },
    surface: "always-loaded-instruction",
    consumers: {
      claude: { state: "supported", mechanism: "claude-global-instruction" },
      codex: { state: "supported", mechanism: "codex-global-instruction" },
      cursor: {
        state: "unsupported",
        reason: "No managed instruction surface.",
      },
    },
    verification: { state: "unverified" },
    ...overrides,
  };
}

const activeConsumers = (
  surface: unknown,
): Readonly<Record<string, unknown>> | undefined => {
  if (surface === "always-loaded-instruction") {
    return undefined;
  }
  if (surface === "conditional-skill") {
    return {
      claude: { state: "supported", mechanism: "claude-user-skill" },
      codex: { state: "supported", mechanism: "codex-user-skill" },
      cursor: { state: "supported", mechanism: "cursor-user-skill" },
    };
  }
  return {
    claude: {
      state: "unsupported",
      reason: "Repository control does not use an agent adapter.",
    },
    codex: {
      state: "unsupported",
      reason: "Repository control does not use an agent adapter.",
    },
    cursor: {
      state: "unsupported",
      reason: "Repository control does not use an agent adapter.",
    },
  };
};

function active(overrides: TestInvariant = {}): TestInvariant {
  const consumers = activeConsumers(overrides.surface ?? "hook");
  return candidate({
    lifecycle: "active",
    controlKind: "enforceable",
    surface: "hook",
    approval: {
      approvedBy: "Sebastien",
      approvedAt: "2026-09-02T00:00:00.000Z",
    },
    oracle: {
      ...oracle,
    },
    sources: [source(firstPullRequest), source(secondPullRequest)],
    ...(consumers === undefined ? {} : { consumers }),
    ...overrides,
  });
}

function registry(...invariants: readonly TestInvariant[]): InvariantRegistry {
  return parseInvariantRegistry({ version: 1, invariants });
}

function validationOptions(
  inspectOracle: ValidationOptions["inspectOracle"] = (
    _path,
  ): OracleInspection => ({
    discovered: true,
    kind: "regular-file",
    tracked: true,
  }),
): ValidationOptions {
  return { inspectOracle, repositoryRoot: "/repository" };
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
  oracle,
  registry,
  secondPullRequest,
  source,
  type TestInvariant,
  validationOptions,
  verifiedVerification,
};
export { validateInvariantRegistry } from "./invariant-registry-contract.ts";
