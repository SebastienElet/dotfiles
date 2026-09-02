const cspellInstallCommand =
  "npm install --global cspell@10.2.0 @cspell/dict-fr-fr@2.3.2";
const cspellDictionaryLinkCommand =
  'HOME="$test_home" cspell link add @cspell/dict-fr-fr';
const cspellTraceCommand =
  'trace=$(HOME="$test_home" cspell trace --config "$test_home/cspell.json" --dictionary-path full --all rclone vient)';
const cspellRunBlockStart =
  "      - name: Check configuration and user dictionary\n        run: |\n";
const cspellJobStart = "  cspell:\n";
const cspellLintCommandStart =
  'HOME="$test_home" cspell lint --config "$test_home/cspell.json" \\';
const frenchDictionaryGrep = String.raw`printf '%s\n' "$trace" | grep -F "@cspell/dict-fr-fr"`;
const userDictionaryGrep = String.raw`printf '%s\n' "$trace" | grep -F "$test_home/.config/cspell/user.txt"`;
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

const extractCspellJob = (workflow: string): string | undefined => {
  const start = workflow.indexOf(cspellJobStart);
  if (start === -1 || workflow.includes(cspellJobStart, start + 1)) {
    return undefined;
  }
  const remaining = workflow.slice(start + cspellJobStart.length);
  const nextJob = remaining.search(/^ {2}[a-z][\w-]*:\n/mu);
  return remaining.slice(0, nextJob === -1 ? undefined : nextJob);
};

const executableRunLines = (runBlock: string): string[] =>
  runBlock
    .split("\n")
    .filter((line) => line.trim() !== "" && !line.trimStart().startsWith("#"));

const cspellOrderFindings = (
  commands: readonly string[],
): readonly string[] => {
  const install = commands.indexOf(cspellInstallCommand);
  const link = commands.indexOf(cspellDictionaryLinkCommand);
  const trace = commands.indexOf(cspellTraceCommand);
  const frenchGrep = commands.indexOf(frenchDictionaryGrep);
  const userGrep = commands.indexOf(userDictionaryGrep);
  const lint = commands.indexOf(cspellLintCommandStart);
  return [
    ...(install === -1 ? ["CSpell installs the French dictionary"] : []),
    ...(link > install
      ? []
      : ["CSpell links the French dictionary after installation"]),
    ...(trace > link ? [] : ["CSpell traces the linked dictionaries"]),
    ...(frenchGrep > trace
      ? []
      : ["CSpell trace confirms the French dictionary"]),
    ...(userGrep > frenchGrep
      ? []
      : ["CSpell trace confirms the user dictionary"]),
    ...(lint > userGrep ? [] : ["CSpell traces dictionaries before linting"]),
  ];
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
    if (line.trimStart().startsWith("#")) {
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
  const job = extractCspellJob(workflow);
  const runBlock = extractCspellRunBlock(workflow);
  if (job === undefined || runBlock === undefined) {
    return ["CSpell has one active YAML job and run block"];
  }
  const commands = executableRunLines(runBlock);
  const cspellLintCommand = extractCspellLintCommand(runBlock);

  return [
    ...(job.includes("    if: false\n") ||
    /^ {4}if: \$\{\{ false \}\}$/mu.test(job)
      ? ["CSpell job is active"]
      : []),
    ...cspellOrderFindings(commands),
    ...(commands.some((command) => command.trim() === "exit 0")
      ? ["CSpell run block has no early success"]
      : []),
    ...(cspellLintCommand !== undefined &&
    invariantRegistryCspellPaths.every((path) =>
      cspellLintCommand.includes(path),
    )
      ? []
      : ["CSpell lint command includes invariant registry sources"]),
  ];
};

export { cspellWorkflowFindings, invariantRegistryCspellPaths };
