import { join } from "node:path";
import { z } from "zod";

const handlerSchema = z
  .looseObject({
    command: z.string().optional(),
    args: z.array(z.string()).readonly().optional(),
    type: z.string().optional(),
  })
  .readonly();
const groupSchema = z
  .looseObject({ hooks: z.array(handlerSchema).readonly() })
  .readonly();
const hooksSchema = z
  .record(z.string(), z.array(z.unknown()).readonly())
  .readonly();
type Handler = Readonly<
  Pick<z.infer<typeof handlerSchema>, "command" | "args" | "type">
>;
type Agent = "claude" | "codex" | "cursor";
type HookRoots = Readonly<{
  repository: string;
  home: string;
  aliases?: readonly Readonly<{ repository: string; home: string }>[];
}>;

function quoted(path: string): string {
  return `'${path.replaceAll("'", String.raw`'\''`)}'`;
}

function rememQuoted(path: string): string {
  return /^[a-zA-Z0-9/._-]+$/u.test(path) ? path : quoted(path);
}

function commands(
  repository: string,
  home: string,
  agent: Agent,
): readonly string[] {
  const arnes = quoted(join(home, ".local/bin/arnes"));
  const measurementAgent = agent === "claude" ? "claude-code" : agent;
  const common = [`${arnes} measure hook --agent ${measurementAgent}`];
  if (agent === "cursor") {
    return common;
  }
  const host = agent === "claude" ? "claude-code" : "codex-cli";
  return [
    ...common,
    `${arnes} output-discipline`,
    `${quoted(join(home, ".local/bin/agent-memory"))} hook --agent ${agent}`,
    join(home, ".local/bin/agent-handoff"),
    join(repository, "tooling/agent-handoff"),
    join(repository, "scripts/agent_handoff"),
    join(repository, "tooling/agent-handoff/target/release/agent-handoff"),
    ...(agent === "claude" ? ["$HOME/.claude/hooks/agent_handoff"] : []),
    quoted(join(repository, "tooling/format-edited-file")),
    ...["context", "session-init", "observe", "summarize"].flatMap(
      (subcommand) =>
        ["remem", "remem-hook"].map(
          (binary) =>
            `${rememQuoted(join(home, ".local/bin", binary))} ${subcommand} --host ${host}`,
        ),
    ),
    `${rememQuoted(join(home, ".local/bin/remem"))} rules eval --host ${host}`,
  ];
}

function owned(
  handler: Handler,
  expected: readonly string[],
  nested: boolean,
): boolean {
  return (
    handler.command !== undefined &&
    expected.includes(handler.command) &&
    (handler.args === undefined || handler.args.length === 0) &&
    (!nested || handler.type === "command")
  );
}

function withoutOwnedHooks(
  value: unknown,
  agent: Agent,
  roots: HookRoots,
): unknown {
  const expected = [
    ...commands(roots.repository, roots.home, agent),
    ...(roots.aliases ?? []).flatMap((alias) =>
      commands(alias.repository, alias.home, agent),
    ),
  ];
  const hooks = hooksSchema.parse(value);
  return Object.fromEntries(
    Object.entries(hooks).flatMap(
      ([event, entries]: readonly [string, readonly unknown[]]) => {
        const retained = entries.flatMap((entry) => {
          if (agent === "cursor") {
            return owned(handlerSchema.parse(entry), expected, false)
              ? []
              : [entry];
          }
          const group = groupSchema.parse(entry);
          const handlers = group.hooks.filter(
            (handler: Handler) => !owned(handler, expected, true),
          );
          if (handlers.length === group.hooks.length) {
            return [entry];
          }
          return handlers.length === 0 ? [] : [{ ...group, hooks: handlers }];
        });
        return retained.length === 0 && entries.length > 0
          ? []
          : [[event, retained]];
      },
    ),
  );
}

export { withoutOwnedHooks };
export type { Agent, HookRoots };
