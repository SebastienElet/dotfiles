import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);
const mcpCommand =
  'export PATH="$HOME/.local/bin:$HOME/.volta/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"; exec "$HOME/.local/bin/remem" mcp';

function fixture(): ReturnType<typeof createDeploymentFixture> {
  const context = createDeploymentFixture("clean-config");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    JSON.stringify({
      version: 1,
      skills: [],
      mcp: [
        {
          name: "remem",
          agent: "codex",
          scope: "user",
          command: "/bin/sh",
          args: ["-c", mcpCommand],
        },
        {
          name: "remem",
          agent: "claude",
          scope: "user",
          command: "/bin/sh",
          args: ["-c", mcpCommand],
        },
      ],
    }),
  );
  return context;
}
type Fixture = ReturnType<typeof fixture>;

function write(
  context: Fixture,
  relativePath: string,
  contents: string,
): string {
  const path = join(context.home, relativePath);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
  return path;
}
function clean(
  context: Fixture,
  args: readonly string[] = ["--apply"],
): ReturnType<typeof runDeploymentHelper> {
  return runDeploymentHelper(context, {
    helper: "clean-deployment.ts",
    arguments: [context.repository, context.home, ...args],
  });
}

test("removes only the installed MCP registration from Claude shared config", () => {
  const context = fixture();
  const document = {
    theme: "dark",
    token: "keep-private",
    mcpServers: {
      remem: { command: "/bin/sh", args: ["-c", mcpCommand], type: "stdio" },
      foreign: { command: "foreign" },
    },
  };
  const path = write(context, ".claude.json", JSON.stringify(document));
  const inspection = clean(context, []);
  expect(inspection.exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toBe(JSON.stringify(document));
  expect(inspection.stdout).not.toContain("keep-private");
  expect(clean(context).exitCode).toBe(0);
  expect(JSON.parse(readFileSync(path, "utf8"))).toEqual({
    ...document,
    mcpServers: { foreign: document.mcpServers.foreign },
  });
  expect(clean(context).exitCode).toBe(0);
});

test("preserves a foreign MCP registration using the same name", () => {
  const context = fixture();
  const original = JSON.stringify({
    mcpServers: { remem: { command: "foreign", args: [] } },
  });
  const path = write(context, ".claude.json", original);
  expect(clean(context).exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toBe(original);
});

test("removes the installed Codex MCP namespace and attached permissions, preserving policy and unrelated TOML bytes", () => {
  const context = fixture();
  const retained =
    '# personal\nmodel = "keep"\n[features]\nmemories = false\n[mcp_servers.foreign]\ncommand = "keep"\n';
  const owned = `\n[mcp_servers.remem]\ncommand = "/bin/sh"\nargs = ${JSON.stringify(["-c", mcpCommand])}\n[mcp_servers.remem.tools.save_memory]\npermission = "allow"\n`;
  const path = write(context, ".codex/config.toml", retained + owned);
  expect(clean(context).exitCode).toBe(0);
  expect(Bun.TOML.parse(readFileSync(path, "utf8"))).toEqual(
    Bun.TOML.parse(retained),
  );
  expect(readFileSync(path, "utf8")).toContain(retained);
});

test("preserves a foreign Codex server with the same namespace", () => {
  const context = fixture();
  const original = '[mcp_servers.remem]\ncommand = "foreign"\n';
  const path = write(context, ".codex/config.toml", original);
  expect(clean(context).exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toBe(original);
});

test.each([
  [".claude/settings.json", "{broken"],
  [".codex/config.toml", "[broken"],
])(
  "refuses malformed shared configuration %s without deleting deployed files",
  (relativePath, content) => {
    const context = fixture();
    const path = write(context, relativePath, content);
    const generated = write(
      context,
      ".codex/AGENTS.md",
      "keep until inspectable\n",
    );
    const result = clean(context);
    expect(result.exitCode).not.toBe(0);
    expect(readFileSync(path, "utf8")).toBe(content);
    expect(readFileSync(generated, "utf8")).toBe("keep until inspectable\n");
  },
);

test("refuses a symlinked shared config without writing through it", () => {
  const context = fixture();
  const protectedPath = join(context.root, "foreign.json");
  writeFileSync(protectedPath, '{"hooks":{}}');
  mkdirSync(join(context.home, ".claude"));
  symlinkSync(protectedPath, join(context.home, ".claude/settings.json"));
  expect(clean(context).exitCode).not.toBe(0);
  expect(readFileSync(protectedPath, "utf8")).toBe('{"hooks":{}}');
});

test("does not silently remove TOML text mistaken for a table header inside a user string", () => {
  const context = fixture();
  const original = `note = '''\n[mcp_servers.remem]\nkeep\n'''\n[mcp_servers.remem]\ncommand = "/bin/sh"\nargs = ${JSON.stringify(["-c", mcpCommand])}\n`;
  const path = write(context, ".codex/config.toml", original);
  const result = clean(context);
  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(path, "utf8")).toBe(original);
});

test("removes exact deployed Git includes while preserving other config values", () => {
  const context = fixture();
  const path = write(
    context,
    ".gitconfig",
    "[user]\n\tname = Keep\n[include]\n\tpath = ~/.config/git/config.delta\n\tpath = ~/foreign\n\tpath = ~/.config/git/config.delta\n",
  );
  expect(clean(context, []).exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toContain("config.delta");
  expect(clean(context).exitCode).toBe(0);
  const result = Bun.spawnSync([
    "git",
    "config",
    "--file",
    path,
    "--get-all",
    "include.path",
  ]);
  expect(result.stdout.toString()).toBe("~/foreign\n");
  expect(readFileSync(path, "utf8")).toContain("name = Keep");
  expect(clean(context).exitCode).toBe(0);
});

test("refuses invalid Git config before deleting other deployments", () => {
  const context = fixture();
  const path = write(context, ".gitconfig", "[broken\n");
  const generated = write(context, ".codex/AGENTS.md", "keep\n");
  expect(clean(context).exitCode).not.toBe(0);
  expect(readFileSync(path, "utf8")).toBe("[broken\n");
  expect(readFileSync(generated, "utf8")).toBe("keep\n");
});

test.each(["claude", "codex", "cursor"])(
  "removes owned %s hook commands while preserving adjacent handlers and memory policy",
  (agent) => {
    const context = fixture();
    const suffix = agent === "claude" ? "claude-code" : agent;
    const owned = {
      type: "command",
      command: `'${context.home}/.local/bin/arnes' measure hook --agent ${suffix}`,
    };
    const neighbor = { type: "command", command: "foreign", args: ["keep"] };
    const entries =
      agent === "cursor"
        ? [owned, neighbor]
        : [{ matcher: "keep", hooks: [owned, neighbor] }];
    const relativePath =
      agent === "claude" ? ".claude/settings.json" : `.${agent}/hooks.json`;
    const path = write(
      context,
      relativePath,
      JSON.stringify({ autoMemoryEnabled: false, hooks: { Event: entries } }),
    );
    expect(clean(context).exitCode).toBe(0);
    expect(JSON.parse(readFileSync(path, "utf8"))).toEqual({
      autoMemoryEnabled: false,
      hooks: {
        Event:
          agent === "cursor"
            ? [neighbor]
            : [{ matcher: "keep", hooks: [neighbor] }],
      },
    });
  },
);

test("recognizes pinned remem slim/full hook commands and preserves similar foreign commands", () => {
  const context = fixture();
  const owned = ["remem", "remem-hook"].flatMap((binary) =>
    ["context", "session-init", "summarize", "observe"].map((operation) => ({
      type: "command",
      command: `${context.home}/.local/bin/${binary} ${operation} --host claude-code`,
    })),
  );
  const neighbor = {
    type: "command",
    command: `${context.home}/.local/bin/remem-hook context --host foreign`,
  };
  const path = write(
    context,
    ".claude/settings.json",
    JSON.stringify({ hooks: { Event: [{ hooks: [...owned, neighbor] }] } }),
  );
  expect(clean(context).exitCode).toBe(0);
  expect(JSON.parse(readFileSync(path, "utf8"))).toEqual({
    hooks: { Event: [{ hooks: [neighbor] }] },
  });
});

test("preserves extra arguments on a command matching an owned hook", () => {
  const context = fixture();
  const original = JSON.stringify({
    hooks: {
      Stop: [
        {
          hooks: [
            {
              type: "command",
              command: `${context.home}/.local/bin/agent-handoff`,
              args: ["foreign"],
            },
          ],
        },
      ],
    },
  });
  const path = write(context, ".claude/settings.json", original);
  expect(clean(context).exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toBe(original);
});

test("preserves an unrelated integer lexeme beyond native number precision", () => {
  const context = fixture();
  const original = `{"privateId":9007199254740993,"mcpServers":{"remem":{"command":"/bin/sh","args":${JSON.stringify(["-c", mcpCommand])}}}}`;
  const path = write(context, ".claude.json", original);
  expect(clean(context).exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toContain("9007199254740993");
  expect(readFileSync(path, "utf8")).not.toContain('"remem"');
});

test("does not expose malformed private configuration contents in diagnostics", () => {
  const context = fixture();
  write(context, ".claude.json", '"private-sensitive-content" broken');
  const result = clean(context);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr + result.stdout).not.toContain(
    "private-sensitive-content",
  );
});

test("reports a native Git update failure while preserving the locked config", () => {
  const context = fixture();
  const original = "[include]\npath = ~/.config/git/config.delta\n";
  const path = write(context, ".gitconfig", original);
  mkdirSync(`${path}.lock`);
  const result = clean(context);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Cannot remove owned Git includes");
  expect(readFileSync(path, "utf8")).toBe(original);
});
