import { afterEach, expect, test } from "bun:test";
import {
  cleanupRememFixtures,
  createRememFixture,
  readTrace,
  repository,
  runUtility,
} from "./remem-test-support.ts";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

afterEach(cleanupRememFixtures);

const toolsSource = readFileSync(
  join(repository, "home", ".codex", "remem-mcp-tools.toml"),
  "utf8",
);
const configurationSchema = z.looseObject({
  mcp_servers: z.looseObject({
    remem: z.looseObject({ tools: z.record(z.string(), z.unknown()) }),
  }),
});

function writeConfiguration(home: string, content: string): string {
  const path = join(home, ".codex", "config.toml");
  mkdirSync(join(home, ".codex"), { recursive: true });
  writeFileSync(path, content);
  return path;
}

function installedTools(path: string): unknown {
  return configurationSchema.parse(Bun.TOML.parse(readFileSync(path, "utf8")))
    .mcp_servers.remem.tools;
}

function install(
  fixture: Readonly<ReturnType<typeof createRememFixture>>,
): ReturnType<typeof runUtility> {
  return runUtility(fixture, [
    "remem-codex-install.ts",
    repository,
    fixture.home,
  ]);
}

const expectedTools = configurationSchema.parse(
  Bun.TOML.parse(`[mcp_servers.remem]\ncommand = "remem"\n\n${toolsSource}`),
).mcp_servers.remem.tools;

test("registers the server and appends the tool permissions once on replay", () => {
  const fixture = createRememFixture();
  const path = writeConfiguration(
    fixture.home,
    'model = "gpt"\n\n[mcp_servers.remem]\ncommand = "remem"\n',
  );

  expect(install(fixture).exitCode).toBe(0);
  const first = readFileSync(path, "utf8");
  const [disable, add, get] = readTrace(fixture).map(({ arguments: args }) =>
    args.join(" "),
  );
  expect(disable).toBe("features disable memories");
  expect(add).toStartWith("mcp add remem -- /bin/sh -c ");
  expect(get).toBe("mcp get remem");

  expect(install(fixture).exitCode).toBe(0);

  expect(readFileSync(path, "utf8")).toBe(first);
  expect(installedTools(path)).toEqual(expectedTools);
  expect(first).toContain('model = "gpt"');
});

test("replaces stale tool permissions with the tracked ones", () => {
  const fixture = createRememFixture();
  const path = writeConfiguration(
    fixture.home,
    '[mcp_servers.remem]\ncommand = "remem"\n\n[mcp_servers.remem.tools.save_memory]\napproval_mode = "prompt"\n\n[other]\nkept = true\n',
  );

  expect(install(fixture).exitCode).toBe(0);

  expect(installedTools(path)).toEqual(expectedTools);
  expect(readFileSync(path, "utf8")).toContain("[other]\nkept = true");
});

test("rejects an unparsable configuration without rewriting it", () => {
  const fixture = createRememFixture();
  const path = writeConfiguration(fixture.home, "[mcp_servers.remem\n");

  const result = install(fixture);

  expect(result.exitCode).toBe(1);
  expect(readFileSync(path, "utf8")).toBe("[mcp_servers.remem\n");
});

test("fails when the server cannot be read back", () => {
  const fixture = createRememFixture();
  writeConfiguration(fixture.home, '[mcp_servers.remem]\ncommand = "remem"\n');

  const result = runUtility(
    fixture,
    ["remem-codex-install.ts", repository, fixture.home],
    { FAKE_FAIL_ON: "mcp get" },
  );

  expect(result.exitCode).toBe(1);
});
