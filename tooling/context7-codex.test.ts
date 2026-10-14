import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  linkSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import {
  installContext7,
  migrateContext7Configuration,
} from "./context7-codex.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const homes: string[] = [];
const readableMode = 0o644;
const permissionMask = 0o777;
const privateFileMode = 0o600;
const privateDirectoryMode = 0o700;
type Fixture = Readonly<{ home: string; path: string; directory: string }>;

afterEach(() => {
  for (const home of homes.splice(0)) {
    rmSync(home, { recursive: true, force: true });
  }
});

function fixture(original?: string): Fixture {
  const home = mkdtempSync(join(tmpdir(), "context7-codex-test-"));
  homes.push(home);
  const directory = join(home, ".codex");
  mkdirSync(directory);
  const path = join(directory, "config.toml");
  if (original !== undefined) {
    writeFileSync(path, original);
  }
  return { home, path, directory };
}

function localServer(settings: Readonly<Record<string, unknown>> = {}): string {
  return z.string().parse(
    Bun.TOML.stringify({
      mcp_servers: {
        context7: {
          command: "npx",
          args: ["-y", "@upstash/context7-mcp@4.4.1"],
          ...settings,
        },
      },
    }),
  );
}

test("replaces the npm process chain with authenticated native HTTP", () => {
  const original = z.string().parse(
    Bun.TOML.stringify({
      model: "example-model",
      mcp_servers: {
        context7: {
          command: "npx",
          args: ["-y", "@upstash/context7-mcp", "--api-key", "fixture-secret"],
          enabled: false,
          tool_timeout_sec: 45,
          enabled_tools: ["query-docs"],
        },
        other: { command: "other-server", args: ["preserved"] },
      },
    }),
  );
  expect(Bun.TOML.parse(migrateContext7Configuration(original))).toEqual({
    model: "example-model",
    mcp_servers: {
      context7: {
        url: "https://mcp.context7.com/mcp",
        http_headers: { Authorization: "Bearer fixture-secret" },
        enabled: false,
        tool_timeout_sec: 45,
        enabled_tools: ["query-docs"],
      },
      other: { command: "other-server", args: ["preserved"] },
    },
  });
});

test("moves the existing environment credential to HTTP authentication", () => {
  const migrated = migrateContext7Configuration(
    localServer({ env: { CONTEXT7_API_KEY: "fixture-environment-secret" } }),
  );
  expect(Bun.TOML.parse(migrated)).toEqual({
    mcp_servers: {
      context7: {
        url: "https://mcp.context7.com/mcp",
        http_headers: { Authorization: "Bearer fixture-environment-secret" },
      },
    },
  });
});

test("retains inherited credential references without persisting their values", () => {
  const migrated = migrateContext7Configuration(
    localServer({ env_vars: ["CONTEXT7_API_KEY"] }),
  );
  expect(Bun.TOML.parse(migrated)).toEqual({
    mcp_servers: {
      context7: {
        url: "https://mcp.context7.com/mcp",
        bearer_token_env_var: "CONTEXT7_API_KEY",
      },
    },
  });
});

test("keeps the explicit API key ahead of the environment credential", () => {
  const migrated = migrateContext7Configuration(
    localServer({
      args: ["--yes", "@upstash/context7-mcp", "--api-key", "argument-secret"],
      env: { CONTEXT7_API_KEY: "environment-secret" },
    }),
  );
  expect(migrated).toContain("Bearer argument-secret");
  expect(migrated).not.toContain("environment-secret");
});

test("preserves already deployed HTTP configuration byte for byte", () => {
  const original =
    '[mcp_servers.context7]\nurl = "https://mcp.context7.com/mcp"\nenv_http_headers = { Authorization = "EXISTING_AUTHORIZATION" }\n';
  expect(migrateContext7Configuration(original)).toBe(original);
});

test("refuses malformed and divergent input without replacing the configuration", () => {
  for (const original of [
    'secret = "fixture-secret"\nbroken = [',
    'mcp_servers = "invalid"',
    '[mcp_servers]\ncontext7 = "invalid"',
    localServer({ command: "custom-launcher" }),
    localServer({ args: ["-y", "different-package"] }),
    localServer({ args: ["-y", "@upstash/context7-mcp", "--api-key"] }),
    localServer({ args: ["-y", "@upstash/context7-mcp", "--api-key", ""] }),
    localServer({ args: ["-y", "@upstash/context7-mcp", "--port", "3000"] }),
    localServer({ env: { OTHER_CREDENTIAL: "fixture-secret" } }),
    localServer({ env_vars: ["OTHER_CREDENTIAL"] }),
    '[mcp_servers.context7]\nurl = "https://custom.example/mcp"',
    localServer({ url: "https://mcp.context7.com/mcp" }),
  ]) {
    const { home, path } = fixture(original);
    expect(() => {
      installContext7(home);
    }).toThrow();
    expect(readFileSync(path, "utf8")).toBe(original);
  }
});

test("deploys privately and leaves a repeated deployment untouched", () => {
  const { home, path } = fixture(localServer());
  chmodSync(path, readableMode);
  installContext7(home);
  const first = statSync(path);
  const content = readFileSync(path, "utf8");
  expect(first.mode & permissionMask).toBe(privateFileMode);
  expect(content).not.toContain("npx");
  installContext7(home);
  expect(readFileSync(path, "utf8")).toBe(content);
  expect(statSync(path).ino).toBe(first.ino);
  expect(statSync(path).mtimeMs).toBe(first.mtimeMs);
});

test("sets up native HTTP when Context7 has no previous registration", () => {
  const { home, path } = fixture('model = "preserved-model"\n');
  installContext7(home);
  expect(Bun.TOML.parse(readFileSync(path, "utf8"))).toEqual({
    model: "preserved-model",
    mcp_servers: { context7: { url: "https://mcp.context7.com/mcp" } },
  });
  const empty = fixture();
  rmSync(empty.directory, { recursive: true });
  installContext7(empty.home);
  expect(statSync(empty.directory).mode & permissionMask).toBe(
    privateDirectoryMode,
  );
  expect(statSync(empty.path).mode & permissionMask).toBe(privateFileMode);
});

test("refuses symlinked parents, symlinked files, hard links and non-files", () => {
  const target = fixture(localServer());
  const linkedParent = fixture();
  rmSync(linkedParent.directory, { recursive: true });
  symlinkSync(target.directory, linkedParent.directory);
  const linkedFile = fixture();
  symlinkSync(target.path, linkedFile.path);
  const hardLinkedFile = fixture();
  linkSync(target.path, hardLinkedFile.path);
  const nonFile = fixture();
  mkdirSync(nonFile.path);
  const original = readFileSync(target.path, "utf8");
  for (const { home } of [linkedParent, linkedFile, hardLinkedFile, nonFile]) {
    expect(() => {
      installContext7(home);
    }).toThrow();
  }
  expect(readFileSync(target.path, "utf8")).toBe(original);
});

test("the CLI fails without exposing a malformed configuration or its secrets", () => {
  const { home, path } = fixture('key = "fixture-secret"\nmalformed = [');
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      `${import.meta.dir}/context7-codex.ts`,
      home,
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  expect(result.exitCode).toBe(1);
  expect(result.stdout.toString()).toBe("");
  expect(result.stderr.toString()).toBe("Context7 deployment failed.\n");
  expect(readFileSync(path, "utf8")).toContain("fixture-secret");
});
