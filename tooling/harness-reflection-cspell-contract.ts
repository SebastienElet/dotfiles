const cspellInstallCommand =
  "npm install --global cspell@10.2.0 @cspell/dict-fr-fr@2.3.2";
const cspellDictionaryLinkCommand =
  'HOME="$test_home" cspell link add @cspell/dict-fr-fr';
const cspellTraceCommand =
  'trace=$(HOME="$test_home" cspell trace --config "$test_home/cspell.json" --dictionary-path full --all rclone vient)';
const cspellRunBlockStart =
  "      - name: Check configuration and user dictionary\n        run: |\n";
const cspellLintCommandStart =
  'HOME="$test_home" cspell lint --config "$test_home/cspell.json" \\';
const invariantRegistryCspellPaths = [
  "harness/skills/harness-reflection/SKILL.md",
  "harness/skills/harness-reflection/references/invariant-registry.md",
  "harness/skills/harness-reflection/evals/trigger-queries.json",
  "harness/invariants/registry.json",
] as const;

const extractCspellRunBlock = (workflow: string): string | undefined => {
  const start = workflow.indexOf(cspellRunBlockStart);
  if (
    start === -1 ||
    workflow.includes(cspellRunBlockStart, start + cspellRunBlockStart.length)
  ) {
    return undefined;
  }
  const lines = workflow.slice(start + cspellRunBlockStart.length).split("\n");
  const block: string[] = [];
  for (const line of lines) {
    if (!line.startsWith("          ")) {
      break;
    }
    block.push(line.slice("          ".length));
  }
  return block.length === 0 ? undefined : block.join("\n");
};

const extractCspellLintCommand = (runBlock: string): string | undefined => {
  const lines = runBlock.split("\n");
  const start = lines.indexOf(cspellLintCommandStart);
  if (start === -1 || lines.includes(cspellLintCommandStart, start + 1)) {
    return undefined;
  }
  const command: string[] = [];
  for (let index = start; index < lines.length; index += 1) {
    const line = lines[index];
    if (line === undefined) {
      return undefined;
    }
    if (index > start && !line.startsWith("  ")) {
      return undefined;
    }
    command.push(line);
    if (!line.endsWith("\\")) {
      return command.join("\n");
    }
  }
  return undefined;
};

const cspellWorkflowFindings = (workflow: string): readonly string[] => {
  const runBlock = extractCspellRunBlock(workflow);
  if (runBlock === undefined) {
    return ["CSpell has one explicit YAML run block"];
  }
  const cspellInstallIndex = runBlock.indexOf(cspellInstallCommand);
  const cspellDictionaryLinkIndex = runBlock.indexOf(
    cspellDictionaryLinkCommand,
  );
  const cspellTraceIndex = runBlock.indexOf(cspellTraceCommand);
  const cspellLintIndex = runBlock.indexOf(cspellLintCommandStart);
  const cspellLintCommand = extractCspellLintCommand(runBlock);

  return [
    ...(cspellInstallIndex === -1
      ? ["CSpell installs the French dictionary"]
      : []),
    ...(cspellDictionaryLinkIndex > cspellInstallIndex
      ? []
      : ["CSpell links the French dictionary after installation"]),
    ...(cspellTraceIndex > cspellDictionaryLinkIndex
      ? []
      : ["CSpell traces the linked dictionaries"]),
    ...(cspellLintIndex > cspellTraceIndex
      ? []
      : ["CSpell lints after tracing dictionaries"]),
    ...(runBlock.includes('grep -F "@cspell/dict-fr-fr"')
      ? []
      : ["CSpell trace confirms the French dictionary"]),
    ...(runBlock.includes('grep -F "$test_home/.config/cspell/user.txt"')
      ? []
      : ["CSpell trace confirms the user dictionary"]),
    ...(cspellLintCommand !== undefined &&
    invariantRegistryCspellPaths.every((path) =>
      cspellLintCommand.includes(path),
    )
      ? []
      : ["CSpell lint command includes invariant registry sources"]),
  ];
};

export { cspellWorkflowFindings, invariantRegistryCspellPaths };
