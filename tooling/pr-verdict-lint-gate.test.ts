import { expect, test } from "bun:test";
import { cspellTextPaths } from "./cspell-texts.ts";

const prVerdictPaths = [
  "harness/skills/pr-verdict/SKILL.md",
  "harness/skills/pr-verdict/assets/verdict-template.md",
  "harness/skills/pr-verdict/references/cases.md",
] as const;

test("the text gate spellchecks every pr-verdict source file", () => {
  for (const path of prVerdictPaths) {
    expect(cspellTextPaths).toContain(path);
  }
});
