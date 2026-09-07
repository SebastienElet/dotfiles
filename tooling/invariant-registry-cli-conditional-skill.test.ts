import {
  active,
  marginalAblation,
  registry,
  targetSkillPath,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import {
  cleanup,
  createRegistry,
  repositoryRoot,
  runRegistryCli,
} from "./invariant-registry-cli.test-support.ts";
import { join } from "node:path";

const effectiveAblation = {
  ...marginalAblation,
  conditionalSkillActivation: {
    with: { activated: 6, total: 6 },
    without: { activated: 0, total: 6 },
  },
};

const registryText = (path: string): string =>
  JSON.stringify(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation: effectiveAblation,
        oracle: undefined,
        surface: "conditional-skill",
        targetSkillPath: path,
        verification: verifiedVerification,
      }),
    ),
  );

afterEach(cleanup);

test("CLI validates the shared enforcement-code target and deployments", async () => {
  const path = await createRegistry(registryText(targetSkillPath));
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).toBe(0);
  expect(outcome.stderr).toBe("");
});

test("CLI rejects a conditional target absent from the repository", async () => {
  const path = await createRegistry(
    registryText("harness/skills/missing-skill/SKILL.md"),
  );
  const outcome = await runRegistryCli(path);

  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stderr).toContain("Conditional skill target does not exist");
});

test("bounds target activation evidence to discovery metadata and ADR-036", async () => {
  const adr = await Bun.file(
    join(repositoryRoot, "docs/adr/036-regles-ia-admises-par-ablation.md"),
  ).text();

  expect(adr).toContain("elle devient la skill `enforcement-code`");
  expect(adr).toContain("son déclenchement est mesuré");
});
