import { join } from "node:path";
import { readFileSync } from "node:fs";
import { z } from "zod";

const agentSchema = z.enum(["claude", "codex", "cursor"]);
const installationSchema = z
  .object({ agent: agentSchema, scope: z.enum(["user", "project"]) })
  .readonly();
const skillSchema = z
  .object({
    slug: z.string().regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u),
    installations: z.array(installationSchema).readonly(),
  })
  .readonly();
const manifestSchema = z.looseObject({
  version: z.literal(1),
  skills: z.array(skillSchema).readonly(),
  mcp: z
    .array(
      z
        .looseObject({
          name: z.string().regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u),
          agent: agentSchema,
          scope: z.enum(["user", "project"]),
          command: z.string().min(1),
          args: z.array(z.string()).default([]).readonly(),
        })
        .readonly(),
    )
    .default([])
    .readonly(),
});

type Artifact = Readonly<{ destination: string; source: string | undefined }>;
type McpRegistration = Readonly<
  Pick<
    z.infer<typeof manifestSchema>["mcp"][number],
    "name" | "agent" | "scope" | "command" | "args"
  >
>;

const skillDirectories = {
  claude: ".claude/skills",
  codex: ".agents/skills",
  cursor: ".cursor/skills",
} as const;
const links = [
  ["home/.config/fish", ".config/fish"],
  ["home/.config/nvim", ".config/nvim"],
  ["home/.config/wezterm/wezterm.lua", ".config/wezterm/wezterm.lua"],
  ["home/.config/herdr/config.toml", ".config/herdr/config.toml"],
  ["home/.config/git/config.delta", ".config/git/config.delta"],
  ["home/.config/git/ignore", ".config/git/ignore"],
  ["home/.config/starship.toml", ".config/starship.toml"],
  ["home/.config/tmux/tmux.conf", ".config/tmux/tmux.conf"],
  ["home/cspell.json", "cspell.json"],
  ["home/.config/cspell/user.txt", ".config/cspell/user.txt"],
  ["home/.arnes.yaml", ".arnes.yaml"],
  ["home/.remem/config.toml", ".remem/config.toml"],
  ["harness/skills/remem-memory", ".agents/skills/remem-memory"],
  ["harness/skills/remem-memory", ".claude/skills/remem-memory"],
  ["home/.psqlrc", ".psqlrc"],
  ["harness/AGENTS.md", ".claude/CLAUDE.md"],
  ["harness/SOUL.md", ".claude/SOUL.md"],
  ["harness/USER.md", ".claude/USER.md"],
  [
    "harness/rules/agent-instructions.md",
    ".claude/rules/agent-instructions.md",
  ],
  [
    "harness/rules/memory-governance-cursor.mdc",
    ".cursor/rules/memory-governance-cursor.mdc",
  ],
  ["tooling/arnes/target/release/arnes", ".local/bin/arnes"],
  [
    "tooling/agent-memory/target/release/agent-memory",
    ".local/bin/agent-memory",
  ],
  [
    "tooling/agent-handoff/target/release/agent-handoff",
    ".local/bin/agent-handoff",
  ],
  ["tooling/scrapling-mcp", ".local/bin/scrapling_mcp"],
] as const;
const regularFiles = [
  ".codex/AGENTS.md",
  ".codex/agents/design-claim-auditor.toml",
  ".config/bat/themes/Catppuccin Latte.tmTheme",
  ".config/bat/themes/Catppuccin Mocha.tmTheme",
  "Library/Spelling/fr.aff",
  "Library/Spelling/fr.dic",
  "Library/Spelling/en_US.aff",
  "Library/Spelling/en_US.dic",
] as const;

function deploymentArtifacts(
  repository: string,
  home: string,
): readonly Artifact[] {
  const manifest = manifestSchema.parse(
    Bun.YAML.parse(readFileSync(join(repository, "home/.arnes.yaml"), "utf8")),
  );
  const skills = manifest.skills.flatMap((skill) =>
    skill.installations
      .filter((installation) => installation.scope === "user")
      .map((installation) => ({
        source: join(repository, "harness/skills", skill.slug),
        destination: join(
          home,
          skillDirectories[installation.agent],
          skill.slug,
        ),
      })),
  );
  return [
    ...links.map(([source, destination]) => ({
      source: join(repository, source),
      destination: join(home, destination),
    })),
    ...regularFiles.map((destination) => ({
      source: undefined,
      destination: join(home, destination),
    })),
    ...skills,
  ];
}

const retiredRememCommand =
  'export PATH="$HOME/.local/bin:$HOME/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"; exec "$HOME/.local/bin/remem" mcp';
const retiredRegistrations: readonly McpRegistration[] = [
  "claude",
  "codex",
].map((agent) => ({
  name: "remem",
  agent: agentSchema.parse(agent),
  scope: "user",
  command: "/bin/sh",
  args: ["-c", retiredRememCommand],
}));

function mcpRegistrations(repository: string): readonly McpRegistration[] {
  const current = manifestSchema
    .parse(
      Bun.YAML.parse(
        readFileSync(join(repository, "home/.arnes.yaml"), "utf8"),
      ),
    )
    .mcp.filter((registration) => registration.scope === "user");
  return [...current, ...retiredRegistrations];
}

export { deploymentArtifacts, mcpRegistrations };
export type { Artifact, McpRegistration };
