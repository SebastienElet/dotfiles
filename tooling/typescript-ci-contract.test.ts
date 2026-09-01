import { expect, test } from "bun:test";
import { resolve } from "node:path";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const moonSetup = z
  .object({
    uses: z.literal(
      "moonrepo/setup-toolchain@261c62cb5b0f580c7be7c8cd0f023a2e96756095",
    ),
    with: z.object({ "moon-version": z.literal("2.5.3") }).strict(),
  })
  .strict();
const affectedCondition = "steps.affected.outputs.any == 'true'";
const checkout = z.object({ uses: z.literal("actions/checkout@v5") }).strict();
const fullCheckout = z
  .object({
    uses: z.literal("actions/checkout@v5"),
    with: z
      .object({ "fetch-depth": z.literal(0), filter: z.literal("blob:none") })
      .strict(),
  })
  .strict();
const setupBun = z
  .object({
    uses: z.literal("oven-sh/setup-bun@v2"),
    with: z.object({ "bun-version-file": z.literal("package.json") }).strict(),
  })
  .strict();
const conditionalSetupBun = setupBun.extend({
  if: z.literal(affectedCondition),
});
const install = z
  .object({
    run: z.literal(
      "bun --config=/dev/null --no-env-file install --frozen-lockfile --ignore-scripts",
    ),
  })
  .strict();
const conditionalInstall = install.extend({ if: z.literal(affectedCondition) });
const moonContract = z
  .object({
    if: z.literal(affectedCondition),
    run: z.literal(
      "bun --config=/dev/null --no-env-file test tooling/moon-typescript-gates.test.ts",
    ),
    env: z.object({ MOON_TYPESCRIPT_CONTRACT: z.literal("1") }).strict(),
  })
  .strict();
const selectAffected = z
  .object({
    id: z.literal("affected"),
    run: z.literal(
      `moon query affected > "$RUNNER_TEMP/moon-affected.json"
any="$(jq -r '
  [(.tasks // {} | keys[]), (.projects // {} | .[] | (.tasks // [])[])]
  | any(. == "typescript:lint" or . == "typescript:typecheck" or . == "typescript:format-check")
' "$RUNNER_TEMP/moon-affected.json")"
echo "any=$any" >> "$GITHUB_OUTPUT"
`,
    ),
  })
  .strict();
const moonCiStep = z
  .object({
    if: z.literal(affectedCondition),
    run: z.literal(
      "moon ci --no-actions typescript:lint typescript:typecheck typescript:format-check",
    ),
  })
  .strict();
const mainPush = z.object({ branches: z.tuple([z.literal("main")]) }).strict();
const staticJob = z
  .object({
    name: z.literal("TypeScript static gates"),
    "runs-on": z.literal("ubuntu-latest"),
    steps: z.tuple([
      fullCheckout,
      moonSetup,
      selectAffected,
      conditionalSetupBun,
      conditionalInstall,
      moonContract,
      moonCiStep,
    ]),
  })
  .strict();
const staticWorkflowSchema = z
  .object({
    name: z.literal("TypeScript static gates"),
    on: z
      .object({
        push: mainPush,
        pull_request: z.null(),
      })
      .strict(),
    jobs: z.object({ static: staticJob }).strict(),
  })
  .strict();
const setupVolta = z
  .object({ uses: z.literal("volta-cli/action@v4") })
  .strict();
const verifyRuntime = z
  .object({
    run: z.literal(
      "bun --config=/dev/null --no-env-file tooling/node-version-contract.ts verify-runtime",
    ),
  })
  .strict();
const bunTest = z
  .object({
    run: z.literal("bun --config=/dev/null --no-env-file test --timeout 15000"),
    env: z.object({ SCRAPLING_DOCKER_SMOKE: z.literal("1") }).strict(),
  })
  .strict();
const testJob = z
  .object({
    name: z.literal("Bun tests"),
    "runs-on": z.literal("ubuntu-latest"),
    steps: z.tuple([
      checkout,
      setupVolta,
      setupBun,
      install,
      verifyRuntime,
      bunTest,
    ]),
  })
  .strict();
const testWorkflowSchema = z
  .object({
    name: z.literal("TypeScript"),
    on: z.tuple([z.literal("push"), z.literal("pull_request")]),
    jobs: z.object({ test: testJob }).strict(),
  })
  .strict();
const lintTask = z.object({
  command: z.literal(
    "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
  ),
});
const typecheckTask = z.object({
  command: z.literal("bun --config=/dev/null --no-env-file run typecheck"),
});
const formatCheckTask = z.object({
  command: z.literal("bun run format:typescript:check"),
});
const moonProjectSchema = z.looseObject({
  language: z.literal("unknown"),
  tasks: z
    .object({
      lint: lintTask.extend({ toolchains: z.literal("system") }),
      typecheck: typecheckTask.extend({ toolchains: z.literal("system") }),
      "format-check": formatCheckTask.extend({
        toolchains: z.literal("system"),
      }),
    })
    .strict(),
});
const workflowStepSchema = z.looseObject({
  name: z.string().optional(),
  run: z.string().optional(),
  uses: z.string().optional(),
});
const lintWorkflowSchema = z.looseObject({
  jobs: z.looseObject({
    cspell: z.looseObject({ steps: z.array(workflowStepSchema) }),
  }),
});

test("runs the affected static gates through one Moon job", async (): Promise<void> => {
  const staticWorkflow = await Bun.file(
    resolve(repositoryRoot, ".github/workflows/check-typescript.yml"),
  ).text();
  const testWorkflow = await Bun.file(
    resolve(repositoryRoot, ".github/workflows/test-typescript.yml"),
  ).text();

  expect(
    staticWorkflowSchema.safeParse(Bun.YAML.parse(staticWorkflow)).success,
  ).toBeTrue();
  expect(
    testWorkflowSchema.safeParse(Bun.YAML.parse(testWorkflow)).success,
  ).toBeTrue();
  for (const bypass of [
    staticWorkflow.replace("moon ci", "moon ci || true"),
    staticWorkflow.replace(
      "    name: TypeScript static gates",
      "    name: TypeScript static gates\n    continue-on-error: true",
    ),
    staticWorkflow.replace("typescript:typecheck", ""),
    `defaults:\n  run:\n    shell: "true {0}"\n${staticWorkflow}`,
  ]) {
    expect(
      staticWorkflowSchema.safeParse(Bun.YAML.parse(bypass)).success,
    ).toBeFalse();
  }
});

test("preserves commands on the system toolchain", async (): Promise<void> => {
  const moonProject = Bun.YAML.parse(
    await Bun.file(resolve(repositoryRoot, "moon.yml")).text(),
  );

  expect(moonProjectSchema.safeParse(moonProject).success).toBeTrue();
});

test("does not prepare Bun outside the affected Moon graph", async (): Promise<void> => {
  const lintWorkflow = lintWorkflowSchema.parse(
    Bun.YAML.parse(
      await Bun.file(
        resolve(repositoryRoot, ".github/workflows/lint.yml"),
      ).text(),
    ),
  );

  for (const step of lintWorkflow.jobs.cspell.steps) {
    expect(step.uses).not.toBe("oven-sh/setup-bun@v2");
    expect(step.run?.startsWith("bun ")).not.toBeTrue();
    expect(step.name).not.toBe("Check TypeScript formatting");
  }
});
