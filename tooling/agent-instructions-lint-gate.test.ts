import { expect, test } from "bun:test";
import { cspellTextPaths } from "./cspell-texts.ts";

test("the text gate spellchecks the canonical agent instructions", () => {
  expect(cspellTextPaths).toContain("harness/AGENTS.md");
});
