import type { Agent, HookKind } from "./install-agent-hooks.ts";
import { dirname, join, resolve } from "node:path";
import { lstatSync, readlinkSync } from "node:fs";
import { z } from "zod";

const objectSchema = z.record(z.string(), z.unknown());
const handlerSchema = z
  .looseObject({
    type: z.string().optional(),
    command: z.string().optional(),
  })
  .readonly();
const groupSchema = z
  .looseObject({
    matcher: z.string().optional(),
    hooks: z.array(handlerSchema).readonly(),
  })
  .readonly();
const nestedSchema = z.looseObject({
  hooks: z.record(z.string(), z.array(groupSchema)).default({}),
});
const directSchema = z.looseObject({
  version: z.number().default(1),
  hooks: z.record(z.string(), z.array(handlerSchema)).default({}),
});
const events = {
  claude: [
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PermissionDenied",
    "PostToolUse",
    "PostToolUseFailure",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "StopFailure",
    "PreCompact",
    "PostCompact",
    "SessionEnd",
  ],
  codex: [
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
  ],
  cursor: [
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
    "postCompact",
    "sessionEnd",
  ],
} as const;
const sessionMatcher = "startup|resume|clear|compact";
const outputTimeout = 30;
const formatTimeout = 30;
const nudgeTimeout = 5;
type Handler = z.infer<typeof handlerSchema>;
type Group = z.infer<typeof groupSchema>;
type Options = Readonly<{
  agent: Agent;
  home: string;
  repository: string;
  kinds: readonly HookKind[];
}>;

function quote(command: string): string {
  return `'${command.replaceAll("'", String.raw`'\''`)}'`;
}

function commands(options: Options): Record<HookKind, string> {
  const { agent, home, repository } = options;
  return {
    measurement: `${quote(join(home, ".local/bin/agent-measure"))} hook --agent ${agent === "claude" ? "claude-code" : agent}`,
    "output-discipline": quote(join(home, ".local/bin/output-discipline-hook")),
    handoff: join(home, ".local/bin/agent-handoff"),
    "format-edited-file": quote(join(repository, "tooling/format-edited-file")),
    "semctx-nudge": `${quote(join(repository, "tooling/semctx-nudge"))} --host ${agent}`,
  };
}

function retiredCommands(options: Options): readonly string[] {
  const { agent, home, repository } = options;
  const arnes = quote(join(home, ".local/bin/arnes"));
  const handoff = join(home, ".local/bin/agent-handoff");
  const target = handoffTarget(handoff);
  const host = agent === "claude" ? "claude-code" : "codex-cli";
  const remem = ["remem", "remem-hook"].flatMap((binary) => {
    const path = join(home, ".local/bin", binary);
    const command = /^[\w/.-]+$/u.test(path) ? path : quote(path);
    const operations =
      binary === "remem"
        ? ["context", "session-init", "observe", "summarize", "rules eval"]
        : ["context", "session-init", "observe", "summarize"];
    return operations.map(
      (operation) => `${command} ${operation} --host ${host}`,
    );
  });
  return [
    `${arnes} measure hook --agent ${agent === "claude" ? "claude-code" : agent}`,
    `${arnes} output-discipline`,
    `${join(repository, "tooling/semctx-nudge")} --host ${agent}`,
    target,
    ...remem,
  ];
}

function owned(handler: Handler, command: string): boolean {
  return (
    (handler.type === undefined || handler.type === "command") &&
    handler.command === command
  );
}

function handoffTarget(path: string): string {
  return lstatSync(path, { throwIfNoEntry: false })?.isSymbolicLink() === true
    ? resolve(dirname(path), readlinkSync(path))
    : path;
}

function removable(handler: Handler, managed: readonly string[]): boolean {
  const remem =
    handler.command !== undefined &&
    /\/(?:remem|remem-hook)'? /u.test(handler.command);
  const customized =
    handler.args !== undefined &&
    (!Array.isArray(handler.args) || handler.args.length > 0);
  return (
    !(remem && customized) && managed.some((command) => owned(handler, command))
  );
}

function retainedGroups(
  groups: readonly Group[],
  managed: readonly string[],
): Group[] {
  return groups.flatMap((group) => {
    const hooks = group.hooks.filter((handler) => !removable(handler, managed));
    return hooks.length === 0 && group.hooks.length > 0
      ? []
      : [{ ...group, hooks }];
  });
}

function matched(command: string, matcher: string, timeout: number): Group {
  return { matcher, hooks: [{ type: "command", command, timeout }] };
}

function nestedConfiguration(input: unknown, options: Options): unknown {
  const config = nestedSchema.parse(input);
  const desired = commands(options);
  const managed = [...Object.values(desired), ...retiredCommands(options)];
  const previousHandoff = Object.values(config.hooks)
    .flatMap((groups: readonly Group[]) =>
      groups.flatMap((group) => group.hooks),
    )
    .find(
      (handler) =>
        owned(handler, desired.handoff) ||
        owned(handler, handoffTarget(desired.handoff)),
    );
  const retained = Object.fromEntries(
    Object.entries(config.hooks).map(
      ([event, groups]: readonly [string, readonly Group[]]) => [
        event,
        retainedGroups(groups, managed),
      ],
    ),
  );
  const hooks: Record<string, readonly Group[]> = { ...retained };
  for (const kind of options.kinds) {
    for (const [event, group] of kindGroups(kind, options, previousHandoff)) {
      hooks[event] = [...(hooks[event] ?? []), group];
    }
  }
  return { ...config, hooks };
}

function kindGroups(
  kind: HookKind,
  options: Options,
  previousHandoff?: Handler,
): readonly (readonly [string, Group])[] {
  const command = commands(options)[kind];
  const { agent } = options;
  switch (kind) {
    case "measurement": {
      return events[agent].map(
        (event) => [event, { hooks: [{ type: "command", command }] }] as const,
      );
    }
    case "output-discipline": {
      return [
        ["SessionStart", matched(command, sessionMatcher, outputTimeout)],
      ];
    }
    case "format-edited-file": {
      return [
        [
          "PostToolUse",
          matched(
            command,
            agent === "claude" ? "Edit|Write|MultiEdit" : "apply_patch",
            formatTimeout,
          ),
        ],
      ];
    }
    case "semctx-nudge": {
      return [["SessionStart", matched(command, sessionMatcher, nudgeTimeout)]];
    }
    case "handoff": {
      return [
        ["Stop", { hooks: [handoffHandler(command, agent, previousHandoff)] }],
      ];
    }
    default: {
      throw new Error("Unknown hook kind");
    }
  }
}

function handoffHandler(
  command: string,
  agent: Agent,
  previous?: Handler,
): Handler {
  const removed =
    agent === "claude"
      ? ["async", "asyncRewake", "once", "if"]
      : ["async", "args"];
  const cleaned = Object.fromEntries(
    Object.entries(previous ?? {}).filter(
      ([key]: readonly [string, unknown]) => !removed.includes(key),
    ),
  );
  return handlerSchema.parse({
    ...cleaned,
    type: "command",
    command,
    ...(agent === "claude" ? { args: [] } : {}),
  });
}

function reconcileAgentHooks(input: unknown, options: Options): unknown {
  objectSchema.parse(input);
  if (options.agent !== "cursor") {
    return nestedConfiguration(input, options);
  }
  const config = directSchema.parse(input);
  const desired = commands(options).measurement;
  const managed = [
    desired,
    `${quote(join(options.home, ".local/bin/arnes"))} measure hook --agent cursor`,
  ];
  const retained = Object.fromEntries(
    Object.entries(config.hooks).map(
      ([event, handlers]: readonly [string, readonly Handler[]]) => [
        event,
        handlers.filter(
          (handler) => !managed.some((command) => owned(handler, command)),
        ),
      ],
    ),
  );
  const selected = options.kinds.includes("measurement") ? events.cursor : [];
  const hooks: Record<string, readonly Handler[]> = { ...retained };
  for (const event of selected) {
    hooks[event] = [...(hooks[event] ?? []), { command: desired }];
  }
  return { ...config, hooks };
}

export { reconcileAgentHooks };
