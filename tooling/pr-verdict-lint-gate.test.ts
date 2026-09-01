import { expect, test } from "bun:test";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const workflowPath = resolve(repositoryRoot, ".github/workflows/lint.yml");
const prVerdictPaths = [
  "harness/skills/pr-verdict/SKILL.md",
  "harness/skills/pr-verdict/assets/verdict-template.md",
  "harness/skills/pr-verdict/references/cases.md",
] as const;

const stepBody = (workflow: string, name: string): string => {
  const start = workflow.indexOf(`      - name: ${name}\n`);
  expect(start).toBeGreaterThanOrEqual(0);

  const nextStep = workflow.indexOf("\n      - ", start + 1);
  return workflow.slice(start, nextStep === -1 ? undefined : nextStep);
};

test("the text lint gate spellchecks every pr-verdict source file", async () => {
  const workflow = await Bun.file(workflowPath).text();
  const spellStep = stepBody(
    workflow,
    "Check configuration and user dictionary",
  );

  for (const path of prVerdictPaths) {
    expect(spellStep).toContain(path);
  }
});
