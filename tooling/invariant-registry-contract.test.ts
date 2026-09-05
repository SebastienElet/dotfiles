import {
  type InvariantRecord,
  type InvariantRegistry,
  parseInvariantRegistry,
} from "./invariant-registry-contract.ts";
import {
  type TestInvariant,
  candidate,
} from "./invariant-registry-test-support.ts";
import { expect, expectTypeOf, test } from "bun:test";

const conditionalSkillConsumers = {
  claude: { state: "supported", mechanism: "claude-user-skill" },
  codex: { state: "supported", mechanism: "codex-user-skill" },
  cursor: { state: "supported", mechanism: "cursor-user-skill" },
} as const;
const targetSkillPath = "harness/skills/enforcement-code/SKILL.md";

expectTypeOf<InvariantRegistry["invariants"]>().not.toExtend<
  InvariantRecord[]
>();
expectTypeOf<InvariantRecord["sources"]>().not.toExtend<
  InvariantRecord["sources"][number][]
>();
expectTypeOf<InvariantRecord["scope"]["exceptions"]>().not.toExtend<
  InvariantRecord["scope"]["exceptions"][number][]
>();
expectTypeOf<
  InvariantRecord["scope"]["exceptions"][number]["paths"]
>().not.toExtend<string[]>();
expectTypeOf<InvariantRecord["consumers"]>().toEqualTypeOf<
  Readonly<InvariantRecord["consumers"]>
>();
expectTypeOf<
  Extract<InvariantRecord, { surface: "conditional-skill" }>
>().toExtend<Readonly<{ targetSkillPath: string }>>();
expectTypeOf<
  "targetSkillPath" extends keyof Extract<
    InvariantRecord,
    { surface: Exclude<InvariantRecord["surface"], "conditional-skill"> }
  >
    ? true
    : false
>().toEqualTypeOf<false>();

test("rejects unknown registry versions", () => {
  expect(() =>
    parseInvariantRegistry({ version: 2, invariants: [] }),
  ).toThrow();
});

test("rejects unknown lifecycle values", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), lifecycle: "enforced" }],
    }),
  ).toThrow();
});

test("rejects unknown record fields", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), extra: true }],
    }),
  ).toThrow();
});

test("requires separate Claude, Codex and Cursor declarations", () => {
  const consumers: TestInvariant = {
    claude: { state: "supported", mechanism: "claude-global-instruction" },
    codex: { state: "supported", mechanism: "codex-global-instruction" },
  };

  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), consumers }],
    }),
  ).toThrow();
});

test("requires an explicit target only for conditional skills", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [
        {
          ...candidate(),
          consumers: conditionalSkillConsumers,
          surface: "conditional-skill",
        },
      ],
    }),
  ).toThrow();
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [
        {
          ...candidate(),
          consumers: conditionalSkillConsumers,
          surface: "conditional-skill",
          targetSkillPath,
        },
      ],
    }),
  ).not.toThrow();
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), targetSkillPath }],
    }),
  ).toThrow();
});

test.each([
  "../harness/skills/enforcement-code/SKILL.md",
  "/harness/skills/enforcement-code/SKILL.md",
  "harness/skills/enforcement-code/README.md",
  "harness/skills/Enforcement-Code/SKILL.md",
] as const)(
  "rejects conditional target outside the user-skill shape: %s",
  (path) => {
    expect(() =>
      parseInvariantRegistry({
        version: 1,
        invariants: [
          {
            ...candidate(),
            consumers: conditionalSkillConsumers,
            surface: "conditional-skill",
            targetSkillPath: path,
          },
        ],
      }),
    ).toThrow();
  },
);
