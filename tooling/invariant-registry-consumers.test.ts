import {
  active,
  candidate,
  diagnosticCodes,
  marginalAblation,
  registry,
  validateInvariantRegistry,
  validationOptions,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

const defaultConsumers = {
  claude: { state: "supported", mechanism: "claude-global-instruction" },
  codex: { state: "supported", mechanism: "codex-global-instruction" },
  cursor: {
    state: "unsupported",
    reason: "No managed instruction surface.",
  },
};
const unsupportedConsumers = {
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
const conditionalSkillConsumers = {
  claude: {
    state: "supported" as const,
    mechanism: "claude-user-skill" as const,
  },
  codex: {
    state: "supported" as const,
    mechanism: "codex-user-skill" as const,
  },
  cursor: {
    state: "supported" as const,
    mechanism: "cursor-user-skill" as const,
  },
};
const probabilisticSurfaceCases = [
  {
    consumers: defaultConsumers,
    surface: "always-loaded-instruction",
  },
  {
    consumers: conditionalSkillConsumers,
    surface: "conditional-skill",
  },
  {
    consumers: unsupportedConsumers,
    surface: "project-local-contract",
  },
] as const;

test("rejects a nonexistent consumer adapter", () => {
  expect(() =>
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          claude: { state: "supported", mechanism: "nonexistent-adapter" },
        },
      }),
    ),
  ).toThrow();
});

test("rejects a consumer mechanism owned by another agent", () => {
  expect(() =>
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          claude: {
            state: "supported",
            mechanism: "codex-global-instruction",
          },
        },
      }),
    ),
  ).toThrow();
});

test("rejects Cursor user-skill support for an always-loaded instruction", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        consumers: {
          ...defaultConsumers,
          cursor: {
            state: "supported",
            mechanism: "cursor-user-skill",
          },
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("consumer-surface-mismatch");
});

test("rejects user-skill consumers for an active architectural test", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        consumers: {
          claude: { state: "supported", mechanism: "claude-user-skill" },
          codex: { state: "supported", mechanism: "codex-user-skill" },
          cursor: { state: "supported", mechanism: "cursor-user-skill" },
        },
        surface: "architectural-test",
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("consumer-surface-mismatch");
});

test.each([
  "hook",
  "permission",
  "lint",
  "type",
  "architectural-test",
] as const)(
  "accepts agent-independent consumption for active %s",
  (surface) => {
    const diagnostics = validateInvariantRegistry(
      registry(active({ consumers: unsupportedConsumers, surface })),
      validationOptions(),
    );

    expect(diagnosticCodes(diagnostics)).not.toContain(
      "consumer-surface-mismatch",
    );
  },
);

for (const testCase of probabilisticSurfaceCases) {
  test(`has a closed consumer projection for active ${testCase.surface}`, () => {
    const { consumers, surface } = testCase;
    const diagnostics = validateInvariantRegistry(
      registry(
        active({
          consumers,
          controlKind: "probabilistic",
          marginalAblation,
          oracle: undefined,
          surface,
          verification: verifiedVerification,
        }),
      ),
      validationOptions(),
    );

    expect(diagnosticCodes(diagnostics)).not.toContain(
      "consumer-surface-mismatch",
    );
  });
}

test.each([
  {
    expected: true,
    statement: "Different conditional guidance.",
  },
  {
    expected: false,
    statement: marginalAblation.candidateTextExact,
  },
] as const)(
  "conditional candidate text statement mismatch is $expected",
  ({ expected, statement }) => {
    const diagnostics = validateInvariantRegistry(
      registry(
        active({
          consumers: conditionalSkillConsumers,
          controlKind: "probabilistic",
          marginalAblation,
          oracle: undefined,
          statement,
          surface: "conditional-skill",
          verification: verifiedVerification,
        }),
      ),
      validationOptions(),
    );

    expect(
      diagnosticCodes(diagnostics).includes(
        "conditional-skill-statement-mismatch",
      ),
    ).toBe(expected);
  },
);
