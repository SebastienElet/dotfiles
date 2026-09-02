const claudeConsumerMechanisms = [
  "claude-global-instruction",
  "claude-user-skill",
] as const;
const codexConsumerMechanisms = [
  "codex-global-instruction",
  "codex-user-skill",
] as const;
const cursorConsumerMechanisms = ["cursor-user-skill"] as const;

export {
  claudeConsumerMechanisms,
  codexConsumerMechanisms,
  cursorConsumerMechanisms,
};
