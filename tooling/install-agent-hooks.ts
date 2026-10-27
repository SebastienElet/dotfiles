import {
  accessSync,
  constants,
  readFileSync,
  realpathSync,
  statSync,
} from "node:fs";
import { isAbsolute, join } from "node:path";
import { reconcileAgentHooks } from "./reconcile-agent-hooks.ts";
import { updateHookConfiguration } from "./hook-configuration-file.ts";
import { z } from "zod";

const agentSchema = z.enum(["claude", "codex", "cursor"]);
const absolutePathSchema = z
  .string()
  .refine(isAbsolute, "Expected an absolute path");
const argumentsSchema = z.tuple([
  absolutePathSchema,
  absolutePathSchema,
  agentSchema,
]);
const installationSchema = z
  .object({
    agent: agentSchema,
    scope: z.enum(["user", "project"]),
  })
  .readonly();
const hookSchema = z
  .object({
    id: z.enum([
      "measurement",
      "output-discipline",
      "handoff",
      "format-edited-file",
      "semctx-nudge",
    ]),
    installations: z.array(installationSchema).min(1).readonly(),
  })
  .readonly();
const manifestSchema = z.looseObject({
  version: z.literal(1),
  hooks: z.array(hookSchema),
});
const argumentOffset = 2;
type Agent = z.infer<typeof agentSchema>;
type HookKind = z.infer<typeof hookSchema>["id"];

function hookExecutable(
  repository: string,
  home: string,
  kind: HookKind,
): string {
  switch (kind) {
    case "measurement": {
      return join(home, ".local/bin/agent-measure");
    }
    case "output-discipline": {
      return join(home, ".local/bin/output-discipline-hook");
    }
    case "handoff": {
      return join(home, ".local/bin/agent-handoff");
    }
    case "format-edited-file": {
      return join(repository, "tooling/format-edited-file");
    }
    case "semctx-nudge": {
      return join(repository, "tooling/semctx-nudge");
    }
    default: {
      throw new Error("Unknown hook kind");
    }
  }
}

function installAgentHooks(
  repository: string,
  home: string,
  agent: Agent,
): void {
  const root = realpathSync(repository);
  const manifest = manifestSchema.parse(
    Bun.YAML.parse(readFileSync(join(root, "home/.arnes.yaml"), "utf8")),
  );
  const selected = manifest.hooks.filter((hook) =>
    hook.installations.some(
      (installation) =>
        installation.agent === agent && installation.scope === "user",
    ),
  );
  const kinds = selected.map((hook) => hook.id);
  if (new Set(kinds).size !== kinds.length) {
    throw new Error("Duplicate hook declaration");
  }
  if (agent === "cursor" && kinds.some((kind) => kind !== "measurement")) {
    throw new Error("Unsupported Cursor hook declaration");
  }
  for (const kind of kinds) {
    const executable = hookExecutable(root, home, kind);
    accessSync(executable, constants.X_OK);
    if (!statSync(executable).isFile()) {
      throw new Error(`Hook command must be a file: ${executable}`);
    }
  }
  updateHookConfiguration(home, agent, (config) =>
    reconcileAgentHooks(config, { agent, home, kinds, repository: root }),
  );
}

if (import.meta.main) {
  try {
    const [repository, home, agent] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    installAgentHooks(repository, home, agent);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export type { Agent, HookKind };
