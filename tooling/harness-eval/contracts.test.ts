import { caseSchema, triggerSchema } from "./contracts.ts";
import { describe, expect, test } from "bun:test";
import { at } from "./test-support.ts";
import { evaluate } from "./oracle.ts";
import { loadCases } from "./sources.ts";

describe("behavioral contracts", () => {
  test("loads three resolvable code-search cases without duplicating activation prompts", () => {
    const cases = loadCases(process.cwd());
    expect(cases.map((entry) => entry.definition.id)).toEqual([
      "code-search-structural",
      "code-search-literal",
      "code-search-known-path",
    ]);
    expect(cases[0]?.prompt).toBe(
      "Je découvre ce monorepo, aide-moi à le cartographier",
    );
    expect(cases.every((entry) => entry.prompt.length > 0)).toBe(true);
  });

  test("rejects malformed activation scenarios and missing polarity", () => {
    expect(() =>
      triggerSchema.parse({ skill: "x", version: "1", queries: [] }),
    ).toThrow();
    expect(() =>
      triggerSchema.parse({
        skill: "x",
        version: "1",
        queries: [
          { query: "Find x", should_activate: "false", reason: "literal" },
        ],
      }),
    ).toThrow();
    expect(() =>
      triggerSchema.parse({
        skill: "x",
        version: "1",
        queries: [
          { query: "Find x", should_activate: true, reason: "literal" },
        ],
      }),
    ).toThrow();
  });

  test("rejects an unknown oracle and escaping source paths", () => {
    const first = at(loadCases(process.cwd()), 0).definition;
    expect(() => caseSchema.parse({ ...first, oracle: "unknown" })).toThrow();
    expect(() =>
      caseSchema.parse({
        ...first,
        sources: [{ path: "../private.md", heading: "Rule" }],
      }),
    ).toThrow();
  });
});

test("self-report and empty observations never prove correct behavior", () => {
  for (const oracle of [
    "structural-v1",
    "literal-v1",
    "known-path-v1",
  ] as const) {
    expect(evaluate(oracle, [])).toBe("FAIL");
  }
});

test("structural exploration requires skill read before conceptual search", () => {
  const read = {
    tool: "cat" as const,
    args: [".agents/skills/code-search/SKILL.md"],
    exitCode: 0,
  };
  const search = {
    tool: "colgrep-search" as const,
    args: ["dependencies"],
    exitCode: 0,
  };
  expect(evaluate("structural-v1", [read, search])).toBe("PASS");
  expect(evaluate("structural-v1", [search, read])).toBe("FAIL");
  expect(evaluate("structural-v1", [{ ...read, exitCode: 1 }, search])).toBe(
    "FAIL",
  );
});

test("literal lookup permits skill reading but refuses conceptual over-triggering", () => {
  const literal = {
    tool: "rg" as const,
    args: ["FEATURE_FLAG_DISABLED"],
    exitCode: 0,
  };
  expect(evaluate("literal-v1", [literal])).toBe("PASS");
  expect(evaluate("literal-v1", [{ ...literal, args: ["OTHER"] }])).toBe(
    "FAIL",
  );
  expect(
    evaluate("literal-v1", [
      literal,
      { tool: "colgrep-search", args: ["flags"], exitCode: 0 },
    ]),
  ).toBe("FAIL");
});

test("known path must actually be read without exploratory search", () => {
  const read = {
    tool: "cat" as const,
    args: ["src/auth/session.ts"],
    exitCode: 0,
  };
  expect(evaluate("known-path-v1", [read])).toBe("PASS");
  expect(
    evaluate("known-path-v1", [
      read,
      { tool: "rg", args: ["--files"], exitCode: 0 },
    ]),
  ).toBe("FAIL");
});
