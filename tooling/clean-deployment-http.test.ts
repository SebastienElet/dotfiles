import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);
type Transport = Readonly<{
  command?: string;
  url?: string;
  args?: readonly string[];
  agent?: string;
}>;

function fixture(
  transport: Transport,
): ReturnType<typeof createDeploymentFixture> {
  const context = createDeploymentFixture("clean-http");
  mkdirSync(join(context.repository, "home"));
  writeFileSync(
    join(context.repository, "home/.arnes.yaml"),
    JSON.stringify({
      version: 1,
      skills: [],
      mcp: [{ name: "context7", agent: "codex", scope: "user", ...transport }],
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

test.each([
  ['url = "https://mcp.context7.com/mcp"', true],
  ['url = "https://foreign.example/mcp"', false],
  ['command = "foreign"', false],
  ['url = "https://mcp.context7.com/mcp"\ncommand = "foreign"', false],
  ['url = "https://mcp.context7.com/mcp"\nargs = []', false],
  ['url = "https://mcp.context7.com/mcp"\nenv = {}', false],
  ['url = "https://mcp.context7.com/mcp"\nenv_vars = []', false],
  ['url = "https://mcp.context7.com/mcp"\ncwd = "/foreign"', false],
])("cleans only the matching HTTP registration: %s", (transport, removed) => {
  const context = fixture({ url: "https://mcp.context7.com/mcp" });
  const retained = 'model = "keep"\n[mcp_servers.other]\ncommand = "keep"\n';
  const original = `${retained}[mcp_servers.context7]\n${transport}\n[mcp_servers.context7.http_headers]\nAuthorization = "Bearer private-test-token"\n`;
  const path = write(context, ".codex/config.toml", original);
  const inspection = clean(context, []);
  expect(inspection.exitCode).toBe(0);
  expect(readFileSync(path, "utf8")).toBe(original);
  const result = clean(context);
  expect(result.exitCode).toBe(0);
  expect(result.stdout + result.stderr).not.toContain("private-test-token");
  expect(readFileSync(path, "utf8")).toBe(removed ? retained : original);
  expect(clean(context).exitCode).toBe(0);
});

test.each([
  {},
  { command: "foreign", url: "https://mcp.context7.com/mcp" },
  { url: "https://mcp.context7.com/mcp", args: ["foreign"] },
  { url: "https://mcp.context7.com/mcp", agent: "claude" },
])(
  "refuses ambiguous manifest transport before cleanup: %j",
  (transport: Transport) => {
    const context = fixture(transport);
    const generated = write(context, ".codex/AGENTS.md", "keep\n");
    expect(clean(context).exitCode).not.toBe(0);
    expect(readFileSync(generated, "utf8")).toBe("keep\n");
  },
);
