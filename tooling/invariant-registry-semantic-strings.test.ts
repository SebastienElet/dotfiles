import {
  type TestInvariant,
  candidate,
  registry,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

type SemanticFieldCase = Readonly<{
  name: string;
  overrides: TestInvariant;
}>;

const approval = {
  approvedBy: "Sebastien",
  approvedAt: "2026-09-02T00:00:00.000Z",
};
const oracle = {
  name: "fetch-url-redaction",
  failurePath: "Rejected URLs do not expose credentials.",
  testPath: "tooling/fetch-url-redaction.test.ts",
};
const retirement = {
  retiredAt: "2026-09-02T00:00:00.000Z",
  reason: "Superseded by a stronger invariant.",
};

function supportedConsumer(overrides: TestInvariant = {}): TestInvariant {
  return {
    state: "supported",
    mechanism: "always-loaded-instruction",
    ...overrides,
  };
}

function unsupportedConsumer(overrides: TestInvariant = {}): TestInvariant {
  return {
    state: "unsupported",
    reason: "No managed instruction surface.",
    ...overrides,
  };
}

function consumers(overrides: TestInvariant = {}): TestInvariant {
  return {
    claude: supportedConsumer(),
    codex: supportedConsumer(),
    cursor: unsupportedConsumer(),
    ...overrides,
  };
}

const blankSemanticFields: SemanticFieldCase[] = [
  { name: "invariant identity", overrides: { id: " \t" } },
  { name: "invariant statement", overrides: { statement: " \n" } },
  {
    name: "measurement environment",
    overrides: {
      verification: {
        state: "measured",
        lastRun: {
          outcome: "passed",
          ranAt: "2026-09-02T00:00:00.000Z",
          environment: " \t",
        },
      },
    },
  },
  {
    name: "consumer mechanism",
    overrides: {
      consumers: consumers({
        claude: supportedConsumer({ mechanism: " \t" }),
      }),
    },
  },
  {
    name: "consumer verification environment",
    overrides: {
      consumers: consumers({
        claude: supportedConsumer({ lastVerifiedEnvironment: " \t" }),
      }),
    },
  },
  {
    name: "unsupported consumer reason",
    overrides: {
      consumers: consumers({
        cursor: unsupportedConsumer({ reason: " \t" }),
      }),
    },
  },
  {
    name: "scope exception path",
    overrides: {
      scope: {
        kind: "cross-project",
        exceptions: [{ paths: [" \t"], reason: "Generated file." }],
      },
    },
  },
  {
    name: "scope exception reason",
    overrides: {
      scope: {
        kind: "cross-project",
        exceptions: [{ paths: ["generated"], reason: " \t" }],
      },
    },
  },
  {
    name: "approval identity",
    overrides: { approval: { ...approval, approvedBy: " \t" } },
  },
  {
    name: "oracle name",
    overrides: { oracle: { ...oracle, name: " \t" } },
  },
  {
    name: "oracle failure path",
    overrides: { oracle: { ...oracle, failurePath: " \t" } },
  },
  {
    name: "oracle test path",
    overrides: { oracle: { ...oracle, testPath: " \t" } },
  },
  {
    name: "retirement reason",
    overrides: { retirement: { ...retirement, reason: " \t" } },
  },
  {
    name: "replacement identity",
    overrides: { retirement: { ...retirement, replacedBy: " \t" } },
  },
];

test.each(blankSemanticFields)(
  "rejects a whitespace-only $name",
  ({ overrides }): void => {
    expect(() => registry(candidate(overrides))).toThrow();
  },
);

test("preserves whitespace in a nonblank semantic string", (): void => {
  const statement = "  Rejected URLs never expose credentials.  ";
  const parsed = registry(candidate({ statement }));

  expect(parsed.invariants[0]?.statement).toBe(statement);
});
