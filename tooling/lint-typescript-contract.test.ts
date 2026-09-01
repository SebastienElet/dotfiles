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
