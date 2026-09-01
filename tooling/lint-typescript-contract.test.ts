import { expect, test } from "bun:test";
import { resolve } from "node:path";
import { z } from "zod";

const SUCCESS = 0;
const MAXIMUM_STATEMENTS = 20;
const repositoryRoot = resolve(import.meta.dir, "..");
const configuredRules = {
  "eslint/func-style": ["error", "declaration", { allowArrowFunctions: true }],
  "eslint/max-statements": ["error", MAXIMUM_STATEMENTS],
  "eslint/no-await-in-loop": "off",
  "eslint/no-bitwise": "off",
  "eslint/no-duplicate-imports": ["error", { allowSeparateTypeImports: true }],
  "eslint/no-magic-numbers": [
    "error",
    {
      enforceConst: true,
      ignore: [-1, 0, 1],
      ignoreArrayIndexes: true,
      ignoreNumericLiteralTypes: true,
      ignoreTypeIndexes: true,
    },
  ],
  "eslint/no-ternary": "off",
  "eslint/no-undefined": "off",
  "eslint/no-use-before-define": [
    "error",
    { classes: true, functions: false, variables: true },
  ],
  "eslint/one-var": ["error", "never"],
  "eslint/sort-keys": "off",
  "import/consistent-type-specifier-style": [
    "error",
    "prefer-top-level-if-only-type-imports",
  ],
  "import/no-named-export": "off",
  "import/no-nodejs-modules": "off",
  "import/no-relative-parent-imports": "off",
  "import/prefer-default-export": "off",
  "node/no-process-env": "off",
  "node/no-sync": "off",
  "node/no-top-level-await": "off",
  "oxc/no-async-await": "off",
  "oxc/no-optional-chaining": "off",
  "oxc/no-rest-spread-properties": "off",
  "promise/avoid-new": "off",
  "typescript/parameter-properties": [
    "error",
    { prefer: "parameter-property" },
  ],
  "typescript/promise-function-async": "off",
  "unicorn/consistent-function-scoping": [
    "error",
    { checkArrowFunctions: false },
  ],
  "unicorn/import-style": [
    "error",
    {
      styles: { "node:path": { default: false, named: true } },
    },
  ],
  "unicorn/max-nested-calls": ["error", { max: 5 }],
  "unicorn/no-array-reduce": "off",
  "unicorn/no-null": "off",
  "unicorn/no-process-exit": "off",
  "unicorn/number-literal-case": "off",
} as const;
const configuredOverrides = [
  {
    files: ["tooling/scrapling-container.ts"],
    rules: { "eslint/no-control-regex": "off" },
  },
  {
    files: ["oxfmt.config.ts"],
    rules: { "import/no-default-export": "off" },
  },
] as const;
const ruleSettingSchema = z.union([
  z.literal("off"),
  z.literal("error"),
  z.tuple([z.literal("error"), z.unknown()]).rest(z.unknown()),
]);
const packageSchema = z.looseObject({
  devDependencies: z.looseObject({
    oxlint: z.string().regex(/^\d+\.\d+\.\d+$/u),
    "oxlint-tsgolint": z.string().regex(/^\d+\.\d+\.\d+$/u),
  }),
  scripts: z
    .object({
      "format:typescript": z.literal("bun tooling/format-typescript.ts"),
      "format:typescript:check": z.literal(
        "bun tooling/format-typescript.ts --check",
      ),
      lint: z.literal("bun tooling/lint-typescript.ts"),
      test: z.literal("bun test"),
      typecheck: z.literal("tsc --noEmit"),
    })
    .strict(),
});
const configSchema = z
  .object({
    $schema: z.literal("./node_modules/oxlint/configuration_schema.json"),
    categories: z
      .object({
        correctness: z.literal("error"),
        pedantic: z.literal("error"),
        perf: z.literal("error"),
        restriction: z.literal("error"),
        style: z.literal("error"),
        suspicious: z.literal("error"),
      })
      .strict(),
    options: z
      .object({ maxWarnings: z.literal(SUCCESS), typeAware: z.literal(true) })
      .strict(),
    overrides: z.array(
      z
        .object({
          files: z.array(z.string()).min(1),
          rules: z.record(z.string(), z.literal("off")),
        })
        .strict(),
    ),
    plugins: z.tuple([
      z.literal("eslint"),
      z.literal("import"),
      z.literal("node"),
      z.literal("promise"),
      z.literal("typescript"),
      z.literal("unicorn"),
      z.literal("oxc"),
    ]),
    rules: z.record(z.string(), ruleSettingSchema),
  })
  .strict();
const checkoutStepSchema = z
  .object({ uses: z.literal("actions/checkout@v5") })
  .strict();
const setupVoltaStepSchema = z
  .object({ uses: z.literal("volta-cli/action@v4") })
  .strict();
const setupBunStepSchema = z
  .object({
    uses: z.literal("oven-sh/setup-bun@v2"),
    with: z.object({ "bun-version-file": z.literal("package.json") }).strict(),
  })
  .strict();
const installStepSchema = z
  .object({
    run: z.literal(
      "bun --config=/dev/null --no-env-file install --frozen-lockfile --ignore-scripts",
    ),
  })
  .strict();
const gateJobSchema = (name: string, command: string): z.ZodType =>
  z
    .object({
      name: z.literal(name),
      "runs-on": z.literal("ubuntu-latest"),
      steps: z.tuple([
        checkoutStepSchema,
        setupBunStepSchema,
        installStepSchema,
        z.object({ run: z.literal(command) }).strict(),
      ]),
    })
    .strict();
const testJobSchema = z
  .object({
    name: z.literal("Bun tests"),
    "runs-on": z.literal("ubuntu-latest"),
    steps: z.tuple([
      checkoutStepSchema,
      setupVoltaStepSchema,
      setupBunStepSchema,
      installStepSchema,
      z
        .object({
          run: z.literal(
            "bun --config=/dev/null --no-env-file tooling/node-version-contract.ts verify-runtime",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "cargo build --release --manifest-path tooling/agent-memory/Cargo.toml",
          ),
        })
        .strict(),
      z
        .object({
          run: z.literal(
            "bun --config=/dev/null --no-env-file test --timeout 15000",
          ),
          env: z.object({ SCRAPLING_DOCKER_SMOKE: z.literal("1") }).strict(),
        })
        .strict(),
    ]),
  })
  .strict();
const workflowSchema = z
  .object({
    name: z.literal("TypeScript"),
    on: z.tuple([z.literal("push"), z.literal("pull_request")]),
    jobs: z
      .object({
        lint: gateJobSchema(
          "TypeScript lint",
          "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
        ),
        test: testJobSchema,
        typecheck: gateJobSchema(
          "TypeScript types",
          "bun --config=/dev/null --no-env-file run typecheck",
        ),
      })
      .strict(),
  })
  .strict();

test("pins the strict type-aware Oxlint contract", async (): Promise<void> => {
  const packageJson = packageSchema.parse(
    await Bun.file(resolve(repositoryRoot, "package.json")).json(),
  );
  const config = configSchema.parse(
    await Bun.file(resolve(repositoryRoot, ".oxlintrc.json")).json(),
  );
  const lockfile = await Bun.file(resolve(repositoryRoot, "bun.lock")).text();

  expect(packageJson.devDependencies.oxlint).not.toContain("^");
  expect(packageJson.devDependencies["oxlint-tsgolint"]).not.toContain("^");
  expect(
    packageSchema.safeParse({
      ...packageJson,
      scripts: { ...packageJson.scripts, prelint: "exit 0" },
    }).success,
  ).toBeFalse();
  expect(config.rules).toEqual(configSchema.shape.rules.parse(configuredRules));
  expect(config.overrides).toEqual(
    configSchema.shape.overrides.parse(configuredOverrides),
  );
  expect(
    configSchema.safeParse({ ...config, ignorePatterns: ["tooling/**"] })
      .success,
  ).toBeFalse();
  expect(lockfile).toContain('"oxlint": "1.80.0"');
  expect(lockfile).toContain('"oxlint-tsgolint": "7.0.2001"');
});

test("runs Oxlint and TypeScript as independent Ubuntu gates", async (): Promise<void> => {
  const workflow = await Bun.file(
    resolve(repositoryRoot, ".github/workflows/test-typescript.yml"),
  ).text();
  const parsedWorkflow = Bun.YAML.parse(workflow);

  expect(workflowSchema.safeParse(parsedWorkflow).success).toBeTrue();
  for (const bypass of [
    workflow.replace(
      "run: bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
      "run: bun --config=/dev/null --no-env-file tooling/lint-typescript.ts || true",
    ),
    workflow.replace(
      "name: TypeScript lint",
      "name: TypeScript lint\n    continue-on-error: true",
    ),
    workflow.replace(
      "name: TypeScript lint",
      "name: TypeScript lint\n    if: false",
    ),
    workflow.replace("name: Bun tests", "name: Bun tests\n    if: false"),
    workflow.replace(
      "      - run: cargo build --release --manifest-path tooling/agent-memory/Cargo.toml\n",
      "",
    ),
    `defaults:\n  run:\n    shell: "true {0}"\n${workflow}`,
  ]) {
    expect(
      workflowSchema.safeParse(Bun.YAML.parse(bypass)).success,
    ).toBeFalse();
  }
});
