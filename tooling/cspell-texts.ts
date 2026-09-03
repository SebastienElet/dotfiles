const cspellTextPaths = [
  "home/cspell.json",
  "home/.config/cspell/user.txt",
  "harness/AGENTS.md",
  "harness/skills/harness-reflection/SKILL.md",
  "harness/skills/harness-reflection/references/invariant-registry.md",
  "harness/skills/harness-reflection/evals/trigger-queries.json",
  "harness/skills/harness-reflection/evals/promotion-workflow-results.json",
  "harness/invariants/registry.json",
  "docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md",
  "docs/superpowers/plans/2026-09-02-registre-invariants-harnais.md",
  ".superpowers/sdd/2026-09-02-registre-invariants-harnais/breaker-adjudication-report.md",
  "tooling/invariant-registry-fixtures/pr-206-secret-redaction.json",
  "tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json",
  "tooling/invariant-registry-fixtures/synthetic-local-workflow.json",
  "tooling/invariant-registry-skill-target*.ts",
  "tooling/invariant-registry-cli-conditional-skill.test.ts",
  "tooling/invariant-registry-validation-options.ts",
  ".agents/skills/scripts/SKILL.md",
  "harness/skills/agent-instructions/SKILL.md",
  "harness/skills/agent-instructions/references/maintenance.md",
  "harness/skills/design-claim-audit/SKILL.md",
  "harness/skills/design-claim-audit/evals/trigger-queries.json",
  "harness/skills/design-claim-audit/evals/fixture/*.md",
  "home/.codex/agents/design-claim-auditor.toml",
  "harness/skills/issue-creation/SKILL.md",
  "harness/skills/issue-creation/references/forge-capabilities.md",
  "harness/skills/issue-creation/evals/trigger-queries.json",
  "harness/skills/linear-issue-spec/SKILL.md",
  "harness/skills/linear-start/SKILL.md",
  "harness/skills/linear-sync/SKILL.md",
  "harness/skills/linear-workflow/SKILL.md",
  "harness/skills/linear-workflow/references/bitbucket.md",
  "harness/skills/linear-workflow/references/completion-evidence.md",
  "harness/skills/linear-workflow/references/transports.md",
  "harness/skills/memory-governance/SKILL.md",
  "harness/skills/memory-governance/evals/trigger-queries.json",
  "harness/skills/obsidian-retrieval/SKILL.md",
  "harness/skills/obsidian-retrieval/references/obsidian-cli.md",
  "harness/skills/obsidian-retrieval/evals/trigger-queries.json",
  "harness/skills/pr-feedback/SKILL.md",
  "harness/skills/pr-feedback/evals/trigger-queries.json",
  "harness/skills/pr-verdict/SKILL.md",
  "harness/skills/pr-verdict/assets/verdict-template.md",
  "harness/skills/pr-verdict/references/cases.md",
  "harness/skills/requirements-clarification/SKILL.md",
  "harness/skills/requirements-clarification/evals/trigger-queries.json",
  "docs/requirements-clarification-validation.md",
  "tooling/design-claim-audit.test.ts",
  "tooling/obsidian-retrieval/contract.ts",
  "tooling/obsidian-retrieval/contract.test.ts",
  "tooling/obsidian-retrieval/default-corpus-contract.ts",
  "tooling/obsidian-retrieval/evaluation-contract.ts",
  "tooling/install-hunspell-dictionary*.ts",
  "tooling/deployment-*.ts",
  ".agents/skills/apple-notes/scripts/*.ts",
  "tooling/apple-notes*.ts",
] as const;

const usageErrorExitCode = 2;
const cliArgumentOffset = 2;

const runCspellTextGate = async (argumentsValue: unknown): Promise<number> => {
  if (
    !Array.isArray(argumentsValue) ||
    argumentsValue.length !== 1 ||
    typeof argumentsValue[0] !== "string" ||
    !/\S/u.test(argumentsValue[0])
  ) {
    process.stderr.write("CSpell config path required\n");
    return usageErrorExitCode;
  }
  try {
    const child = Bun.spawn(
      ["cspell", "lint", "--config", argumentsValue[0], ...cspellTextPaths],
      { stderr: "inherit", stdout: "inherit" },
    );
    return await child.exited;
  } catch {
    process.stderr.write("unable to execute CSpell\n");
    return 1;
  }
};

if (import.meta.main) {
  process.exitCode = await runCspellTextGate(Bun.argv.slice(cliArgumentOffset));
}

export { cspellTextPaths, runCspellTextGate };
