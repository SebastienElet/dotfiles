import { expect, test } from "bun:test";
import { resolve } from "node:path";

const workflowPath = resolve(import.meta.dir, "../.github/workflows/lint.yml");

const stepBody = (workflow: string, name: string): string => {
  const start = workflow.indexOf(`      - name: ${name}\n`);
  expect(start).toBeGreaterThanOrEqual(0);

  const nextStep = workflow.indexOf("\n      - ", start + 1);
  return workflow.slice(start, nextStep === -1 ? undefined : nextStep);
};

test("text CI spellchecks the canonical agent instructions", async () => {
  const workflow = await Bun.file(workflowPath).text();
  const spellStep = stepBody(
    workflow,
    "Check configuration and user dictionary",
  );

  expect(spellStep).toContain("harness/AGENTS.md");
});
