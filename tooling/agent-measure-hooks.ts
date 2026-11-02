import { dirname, isAbsolute, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { z } from "zod";

const nestedEvents = [
  "SessionStart",
  "UserPromptSubmit",
  "PreToolUse",
  "PermissionRequest",
  "PostToolUse",
  "PreCompact",
  "PostCompact",
  "SubagentStart",
  "SubagentStop",
  "Stop",
  "SessionEnd",
] as const;
const profiles = [
  {
    agent: "claude-code",
    path: ".claude/settings.json",
    nested: true,
    events: [
      ...nestedEvents,
      "PermissionDenied",
      "PostToolUseFailure",
      "StopFailure",
    ],
  },
  {
    agent: "codex",
    path: ".codex/hooks.json",
    nested: true,
    events: nestedEvents,
  },
  {
    agent: "cursor",
    path: ".cursor/hooks.json",
    nested: false,
    events: [
      "sessionStart",
      "beforeSubmitPrompt",
      "preToolUse",
      "postToolUse",
      "postToolUseFailure",
      "subagentStart",
      "subagentStop",
      "afterAgentResponse",
      "stop",
      "preCompact",
      "sessionEnd",
    ],
  },
] as const;

const objectSchema = z.looseObject({});
const hookListSchema = z.array(z.unknown());
const hookEntriesSchema = z.array(z.tuple([z.string(), hookListSchema]));
type Hooks = Readonly<Record<string, readonly unknown[]>>;
type Settings = Readonly<Record<string, unknown>> & Readonly<{ hooks?: Hooks }>;
type Profile = (typeof profiles)[number];
type Deployment = Readonly<{
  path: string;
  before: string | undefined;
  contents: string;
  mode: number;
}>;
const privateMode = 0o600;
const directoryMode = 0o700;
const permissionMask = 0o777;
const argumentOffset = 2;
const jsonIndentation = 2;
const homeSchema = z.string().min(1).refine(isAbsolute);
const handlerSchema = z.looseObject({
  type: z.literal("command").optional(),
  command: z.string(),
});
const groupSchema = z.looseObject({ hooks: z.array(z.unknown()) });

function quotedExecutable(home: string): string {
  return `'${join(home, ".local/bin/agent-measure").replaceAll("'", String.raw`'\''`)}'`;
}

function preservedObject(value: unknown): Record<string, unknown> {
  objectSchema.parse(value);
  return Object.fromEntries(
    Object.getOwnPropertyNames(value).map((key): readonly [string, unknown] => [
      key,
      Object.getOwnPropertyDescriptor(value, key)?.value,
    ]),
  );
}

function managedCommands(home: string): readonly string[] {
  const executables = [
    quotedExecutable(home),
    join(home, ".local/bin/agent-measure"),
  ];
  const commands = ["claude", "claude-code", "codex", "cursor"].flatMap(
    (agent) =>
      executables.map((executable) => `${executable} hook --agent ${agent}`),
  );
  return [
    ...commands,
    ...executables.map((executable) =>
      claudeOnlyCommand(`${executable} hook --agent claude-code`),
    ),
  ];
}

function claudeOnlyCommand(command: string): string {
  return `if [ -z "\${CURSOR_VERSION:-}" ]; then ${command}; fi`;
}

function isMeasurement(entry: unknown, commands: readonly string[]): boolean {
  const handler = handlerSchema.safeParse(entry);
  return handler.success && commands.includes(handler.data.command);
}

function withoutMeasurementGroup(
  entry: unknown,
  commands: readonly string[],
): unknown[] {
  const group = groupSchema.safeParse(entry);
  if (!group.success) {
    return [entry];
  }
  const hooks = group.data.hooks.filter(
    (handler) => !isMeasurement(handler, commands),
  );
  if (hooks.length === group.data.hooks.length) {
    return [entry];
  }
  return hooks.length === 0 ? [] : [{ ...preservedObject(entry), hooks }];
}

function withoutMeasurement(
  hooks: NonNullable<Settings["hooks"]>,
  profile: Profile,
  home: string,
): Record<string, unknown[]> {
  const commands = managedCommands(home);
  return Object.fromEntries(
    Object.entries(hooks).flatMap(
      ([event, entries]: readonly [string, readonly unknown[]]) => {
        const retained = profile.nested
          ? entries.flatMap((entry) => withoutMeasurementGroup(entry, commands))
          : entries.filter((entry) => !isMeasurement(entry, commands));
        return retained.length === 0 && entries.length > 0
          ? []
          : [[event, retained]];
      },
    ),
  );
}

function parsedSettings(contents: string | undefined, path: string): Settings {
  if (contents === undefined) {
    return {};
  }
  try {
    const settings = preservedObject(JSON.parse(contents));
    if (settings.hooks === undefined) {
      return settings;
    }
    const hooks = Object.fromEntries(
      hookEntriesSchema.parse(Object.entries(preservedObject(settings.hooks))),
    );
    return { ...settings, hooks };
  } catch {
    throw new Error(`Invalid hook settings in ${path}`);
  }
}

function mergedSettings(
  settings: Settings,
  profile: Profile,
  home: string,
): Settings {
  const invocation = `${quotedExecutable(home)} hook --agent ${profile.agent}`;
  const command =
    profile.agent === "claude-code"
      ? claudeOnlyCommand(invocation)
      : invocation;
  const handler = profile.nested
    ? { hooks: [{ type: "command", command }] }
    : { command };
  const hooks = withoutMeasurement(settings.hooks ?? {}, profile, home);
  for (const event of profile.events) {
    hooks[event] = [...(hooks[event] ?? []), handler];
  }
  return {
    ...settings,
    ...(profile.agent === "cursor" && settings.version === undefined
      ? { version: 1 }
      : {}),
    hooks,
  };
}

function deployment(home: string, profile: Profile): Deployment {
  const path = join(home, profile.path);
  const directory = lstatSync(dirname(path), { throwIfNoEntry: false });
  if (directory !== undefined && !directory.isDirectory()) {
    throw new Error(`Expected a regular settings directory: ${dirname(path)}`);
  }
  const metadata = lstatSync(path, { throwIfNoEntry: false });
  if (metadata !== undefined && !metadata.isFile()) {
    throw new Error(`Expected a regular settings file: ${path}`);
  }
  const before =
    metadata === undefined ? undefined : readFileSync(path, "utf8");
  const previous = parsedSettings(before, path);
  const settings = mergedSettings(previous, profile, home);
  const unchanged = JSON.stringify(previous) === JSON.stringify(settings);
  return {
    path,
    before,
    contents:
      unchanged && before !== undefined
        ? before
        : `${JSON.stringify(settings, null, jsonIndentation)}\n`,
    mode: metadata === undefined ? privateMode : metadata.mode & permissionMask,
  };
}

function writeDeployment(plan: Deployment): void {
  if (plan.contents === plan.before) {
    return;
  }
  mkdirSync(dirname(plan.path), { recursive: true, mode: directoryMode });
  const temporary = mkdtempSync(join(dirname(plan.path), ".agent-measure-"));
  const staged = join(temporary, "settings");
  try {
    writeFileSync(staged, plan.contents, { mode: plan.mode });
    renameSync(staged, plan.path);
  } finally {
    rmSync(temporary, { force: true, recursive: true });
  }
}

function deployAgentMeasureHooks(home: string): void {
  const directory = homeSchema.parse(home);
  if (!statSync(directory).isDirectory()) {
    throw new Error("Expected an existing home directory");
  }
  const plans = profiles.map((profile) => deployment(directory, profile));
  for (const plan of plans) {
    writeDeployment(plan);
  }
}

if (import.meta.main) {
  try {
    const [home] = z
      .tuple([homeSchema])
      .parse(process.argv.slice(argumentOffset));
    deployAgentMeasureHooks(home);
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { deployAgentMeasureHooks };
