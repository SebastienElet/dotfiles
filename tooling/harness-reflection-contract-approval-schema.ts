import { z } from "zod";

const approvalContractSchema = z
  .object({
    requiredBeforeMutation: z.literal(true),
    preApprovalState: z.literal("session-local"),
    manifestRequired: z.literal(true),
    manifestTiming: z.literal("present-exact-manifest-before-approval"),
    manifestContents: z.tuple([
      z.literal("exact-paths"),
      z.literal("exact-preimages"),
      z.literal("exact-replacements"),
      z.literal("target-invariant-id"),
      z.literal("exact-target-before-and-after"),
    ]),
    transitionKind: z.literal("derived-from-exact-target-before-and-after"),
    proceduralPrecondition: z.literal(
      "contextual-human-approval-before-attestation",
    ),
    codeAcceptance: z.literal(
      "exact-approval-attestation-without-origin-authentication",
    ),
    authentication: z.literal("not-performed"),
    registryRecordMeaning: z.literal(
      "recorded-attestation-not-independent-proof",
    ),
    agentSelfAssertion: z.literal(
      "procedurally-forbidden-not-machine-detectable",
    ),
    timePressureBypass: z.literal(false),
  })
  .strict();

const consumerMechanismSchema = z
  .object({
    claude: z.tuple([
      z.literal("claude-global-instruction"),
      z.literal("claude-user-skill"),
    ]),
    codex: z.tuple([
      z.literal("codex-global-instruction"),
      z.literal("codex-user-skill"),
    ]),
    cursor: z.tuple([z.literal("cursor-user-skill")]),
  })
  .strict();

const mutationTargetsSchema = z
  .object({
    alwaysLoadedInstruction: z
      .object({
        surface: z.literal("always-loaded-instruction"),
        path: z.literal("harness/AGENTS.md"),
        consumers: z
          .object({
            claude: z.literal("claude-global-instruction"),
            codex: z.literal("codex-global-instruction"),
            cursor: z.literal("unsupported"),
          })
          .strict(),
      })
      .strict(),
    conditionalSkill: z
      .object({
        surface: z.literal("conditional-skill"),
        path: z.literal("harness/skills/harness-reflection/SKILL.md"),
        consumers: z
          .object({
            claude: z.literal("claude-user-skill"),
            codex: z.literal("codex-user-skill"),
            cursor: z.literal("cursor-user-skill"),
          })
          .strict(),
      })
      .strict(),
  })
  .strict();

const consumersContractSchema = z
  .object({
    required: z.tuple([
      z.literal("claude"),
      z.literal("codex"),
      z.literal("cursor"),
    ]),
    declaration: z.literal(
      "independent-supported-mechanism-or-unsupported-reason",
    ),
    supportedMechanisms: consumerMechanismSchema,
    mutationTargets: mutationTargetsSchema,
  })
  .strict();

export { approvalContractSchema, consumersContractSchema };
