import {
  type Immutable,
  caseSchema,
  fingerprintSchema,
  observationSchema,
  sourceSchema,
  textSchema,
} from "./contracts.ts";
import { z } from "zod";

const maximumTimeoutSeconds = 600;
const maximumRunCount = 10;

const tokensSchema = z.strictObject({
  input: z.int().nonnegative(),
  cachedInput: z.int().nonnegative(),
  output: z.int().nonnegative(),
});
const runSchema = z.strictObject({
  status: z.enum(["PASS", "FAIL", "INVALID"]),
  error: z
    .enum([
      "agent-failed",
      "timeout",
      "protocol-invalid",
      "observation-invalid",
      "output-limit",
    ])
    .nullable(),
  observations: z.array(observationSchema),
  tokens: tokensSchema.nullable(),
  toolCalls: z.int().nonnegative().nullable(),
  durationMs: z.number().nonnegative().nullable(),
});
const controlsSchema = z.strictObject({
  sandbox: z.literal("workspace-write"),
  network: z.literal(false),
  tools: z.literal("shell-with-synthetic-cat-rg-fd-colgrep-v1"),
  timeoutSeconds: z.int().positive().max(maximumTimeoutSeconds),
  reasoningEffort: z.enum(["low", "medium", "high"]),
  tokenBudget: z.null(),
});
const reportSchema = z.strictObject({
  schemaVersion: z.literal(1),
  agent: textSchema,
  agentVersion: textSchema,
  model: textSchema,
  date: z.iso.datetime(),
  harness: z.strictObject({
    gitRevision: z.string().regex(/^[a-f0-9]{40}$/u),
    instructionFingerprint: fingerprintSchema,
    skillFingerprint: fingerprintSchema,
    variant: textSchema,
  }),
  runnerRevision: fingerprintSchema,
  environment: z.strictObject({
    platform: textSchema,
    architecture: textSchema,
    bun: textSchema,
  }),
  controls: controlsSchema,
  runCount: z.int().positive().max(maximumRunCount),
  cases: z
    .array(
      z.strictObject({
        definition: caseSchema,
        prompt: textSchema,
        promptFingerprint: fingerprintSchema,
        sources: z
          .array(sourceSchema.extend({ fingerprint: fingerprintSchema }))
          .min(1),
        fixtureRevision: fingerprintSchema,
        runs: z.array(runSchema).min(1),
      }),
    )
    .min(1),
  limitations: z.array(textSchema).min(1),
});
type Report = Immutable<z.infer<typeof reportSchema>>;
type Run = Immutable<z.infer<typeof runSchema>>;
type Controls = Immutable<z.infer<typeof controlsSchema>>;

export {
  tokensSchema,
  runSchema,
  controlsSchema,
  reportSchema,
  type Report,
  type Run,
  type Controls,
};
