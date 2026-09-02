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
