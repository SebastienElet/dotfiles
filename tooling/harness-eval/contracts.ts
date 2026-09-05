import { z } from "zod";

const textSchema = z.string().min(1);
const fingerprintSchema = z.string().regex(/^[a-f0-9]{64}$/u);
const pathSchema = z
  .string()
  .regex(/^[a-zA-Z0-9_.-]+(?:\/[a-zA-Z0-9_.-]+)*$/u)
  .refine(
    (value) => !value.split("/").some((part) => part === "." || part === ".."),
    "Non-relative path",
  );
const triggerDefinitionSchema = z.strictObject({
  skill: textSchema,
  version: z.string(),
  queries: z
    .array(
      z.strictObject({
        query: z.string(),
        should_activate: z.boolean(),
        reason: z.string(),
      }),
    )
    .min(1),
});
const triggerSchema = triggerDefinitionSchema.refine(
  (value: Immutable<z.infer<typeof triggerDefinitionSchema>>) =>
    value.queries.some((query) => query.should_activate) &&
    value.queries.some((query) => !query.should_activate),
  "Both activation polarities are required",
);

const sourceSchema = z.strictObject({
  path: pathSchema,
  heading: textSchema,
});
const oracleSchema = z.enum(["structural-v1", "literal-v1", "known-path-v1"]);
const caseSchema = z.strictObject({
  id: z.string().regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u),
  expected: textSchema,
  sources: z.array(sourceSchema).min(1),
  prompt: z.union([
    z.strictObject({ text: textSchema }),
    z.strictObject({
      triggerFile: pathSchema,
      queryIndex: z.int().nonnegative(),
    }),
  ]),
  fixture: z.literal("code-search-v1"),
  oracle: oracleSchema,
  success: textSchema,
  failure: textSchema,
});
const casesSchema = z
  .array(caseSchema)
  .min(1)
  .refine(
    (values: readonly BehavioralCase[]) =>
      new Set(values.map((value) => value.id)).size === values.length,
    "Duplicate case ID",
  );
const fixtureDefinitionSchema = z.strictObject({
  id: z.literal("code-search-v1"),
  files: z.record(pathSchema, textSchema),
});
const fixtureSchema = fixtureDefinitionSchema.refine(
  (value: Immutable<z.infer<typeof fixtureDefinitionSchema>>) =>
    Object.keys(value.files).length > 0,
  "Empty fixture",
);
const observationSchema = z.strictObject({
  tool: z.enum(["cat", "rg", "fd", "colgrep-search"]),
  args: z.array(z.string()),
  exitCode: z.int().nonnegative(),
});
type Immutable<Value> = {
  readonly [Key in keyof Value]: Immutable<Value[Key]>;
};
type BehavioralCase = Immutable<z.infer<typeof caseSchema>>;
type Trigger = Immutable<z.infer<typeof triggerSchema>>;
type Fixture = Immutable<z.infer<typeof fixtureSchema>>;
type Observation = Immutable<z.infer<typeof observationSchema>>;
type Oracle = z.infer<typeof oracleSchema>;

export {
  type Immutable,
  type Trigger,
  type Fixture,
  textSchema,
  fingerprintSchema,
  pathSchema,
  triggerSchema,
  sourceSchema,
  oracleSchema,
  caseSchema,
  casesSchema,
  fixtureSchema,
  observationSchema,
  type BehavioralCase,
  type Observation,
  type Oracle,
};
