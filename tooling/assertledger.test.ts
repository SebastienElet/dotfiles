import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { installAssertledger } from "./install-assertledger.ts";
import { tmpdir } from "node:os";
import { z } from "zod";

const directories: string[] = [];
const executableMode = 0o755;
const forwardedStatus = 23;
interface Fixture {
  readonly home: string;
  readonly repository: string;
  readonly launcher: string;
  readonly runtime: string;
  readonly cli: string;
  readonly versionFile: string;
}
const host = Bun.spawnSync(
  [
    "node",
    "-p",
    "JSON.stringify({version:process.versions.node,executable:process.execPath})",
  ],
  { cwd: "/" },
);
if (host.exitCode !== 0) {
  throw new Error(host.stderr.toString());
}
const node = z
  .object({ version: z.string(), executable: z.string() })
  .parse(JSON.parse(host.stdout.toString()));

afterEach(() => {
  for (const directory of directories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function fixture(): Fixture {
  const root = mkdtempSync(join(tmpdir(), "assertledger-launcher-"));
  directories.push(root);
  const home = join(root, "home with 'quotes'");
  const repository = join(root, "repository");
  const runtime = join(
    home,
    ".volta/tools/image/node",
    node.version,
    "bin/node",
  );
  mkdirSync(dirname(runtime), { recursive: true });
  symlinkSync(node.executable, runtime);
  mkdirSync(repository);
  writeFileSync(
    join(repository, "package.json"),
    JSON.stringify({ volta: { node: node.version } }),
  );
  const versionFile = join(
    repository,
    "node_modules/@sebastienelet/agent-skills/skills/assertledger/version.env",
  );
  mkdirSync(dirname(versionFile), { recursive: true });
  writeFileSync(versionFile, "ASSERTLEDGER_VERSION=1.4.0\n");
  const cli = join(
    home,
    ".bun/install/global/node_modules/assertledger/dist/cli.js",
  );
  mkdirSync(dirname(cli), { recursive: true });
  writeFileSync(
    join(dirname(dirname(cli)), "package.json"),
    JSON.stringify({ version: "1.4.0", engines: { node: ">=22.15.0" } }),
  );
  writeFileSync(
    cli,
    '#!/usr/bin/env node\nif (process.argv[2] === "--version") console.log("assertledger 1.4.0"); else { console.log(JSON.stringify(process.argv.slice(2))); process.exitCode = 23; }\n',
  );
  chmodSync(cli, executableMode);
  const launcher = join(home, ".local/bin/assertledger");
  return { home, repository, launcher, runtime, cli, versionFile };
}

test("launches the managed Node with a minimal PATH and preserves arguments and status", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  const version = Bun.spawnSync([setup.launcher, "--version"], {
    env: { HOME: setup.home, PATH: "/usr/bin:/bin" },
  });
  expect(version.exitCode).toBe(0);
  expect(version.stdout.toString()).toBe("assertledger 1.4.0\n");
  const result = Bun.spawnSync([setup.launcher, "a b", "", "$HOME", "--json"], {
    env: { HOME: setup.home, PATH: "/usr/bin:/bin" },
  });
  expect(result.exitCode).toBe(forwardedStatus);
  expect(JSON.parse(result.stdout.toString())).toEqual([
    "a b",
    "",
    "$HOME",
    "--json",
  ]);
});

async function installationError(setup: Fixture): Promise<string> {
  try {
    await installAssertledger(setup.repository, setup.home);
  } catch (error) {
    return String(error);
  }
  throw new Error("Installation unexpectedly succeeded");
}

test("reinstallation preserves the launcher inode and rejects divergent files and links", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  const inode = lstatSync(setup.launcher).ino;
  await installAssertledger(setup.repository, setup.home);
  expect(lstatSync(setup.launcher).ino).toBe(inode);
  writeFileSync(setup.launcher, "third-party\n");
  expect(await installationError(setup)).toContain("not a managed launcher");
  expect(lstatSync(setup.launcher).ino).toBe(inode);
  rmSync(setup.launcher);
  symlinkSync("missing-third-party", setup.launcher);
  expect(await installationError(setup)).toContain("not a managed launcher");
  expect(readlinkSync(setup.launcher)).toBe("missing-third-party");
});

test("reinstallation restores owner permissions needed to launch", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  const groupExecutableMode = 0o654;
  chmodSync(setup.launcher, groupExecutableMode);
  await installAssertledger(setup.repository, setup.home);
  const result = Bun.spawnSync(
    ["/bin/bash", "-c", 'exec "$1" --version', "--", setup.launcher],
    { env: { PATH: "/usr/bin:/bin" } },
  );
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toBe("assertledger 1.4.0\n");
});

test("refuses absent and incompatible runtimes or CLI versions without publishing an entry", async () => {
  const setup = fixture();
  rmSync(setup.runtime);
  expect(await installationError(setup)).toContain("ENOENT");
  expect(existsSync(setup.launcher)).toBe(false);
  writeFileSync(setup.runtime, "#!/usr/bin/env bash\nprintf 'v20.0.0\\n'\n", {
    mode: executableMode,
  });
  expect(await installationError(setup)).toContain(
    "does not match project pin",
  );
  rmSync(setup.runtime);
  symlinkSync(node.executable, setup.runtime);
  rmSync(setup.cli);
  expect(await installationError(setup)).toContain(
    "AssertLedger 1.4.0 is required",
  );
  writeFileSync(setup.cli, 'console.log("assertledger 0.0.0");\n');
  expect(await installationError(setup)).toContain(
    "AssertLedger 1.4.0 is required",
  );
  expect(existsSync(setup.launcher)).toBe(false);
});

test("refuses malformed installation inputs and destination parents", async () => {
  const setup = fixture();
  writeFileSync(setup.versionFile, "ASSERTLEDGER_VERSION=latest\n");
  expect(await installationError(setup)).toContain("Invalid string");
  writeFileSync(setup.versionFile, "ASSERTLEDGER_VERSION=1.4.0\n");
  mkdirSync(join(setup.home, ".local"));
  writeFileSync(dirname(setup.launcher), "keep\n");
  expect(await installationError(setup)).toContain("ENOTDIR");
  expect(existsSync(setup.launcher)).toBe(false);
});

test("refreshes an owned launcher when the installed Node pin changes", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  const nextVersion = node.version === "22.16.0" ? "24.0.0" : "22.16.0";
  const nextRuntime = join(
    setup.home,
    ".volta/tools/image/node",
    nextVersion,
    "bin/node",
  );
  mkdirSync(dirname(nextRuntime), { recursive: true });
  writeFileSync(
    nextRuntime,
    `#!/usr/bin/env bash\nif [ "$1" = --version ]; then printf '%s\\n' v${nextVersion}; else exec '${node.executable.replaceAll("'", String.raw`'\''`)}' "$@"; fi\n`,
    { mode: executableMode },
  );
  writeFileSync(
    join(setup.repository, "package.json"),
    JSON.stringify({ volta: { node: nextVersion } }),
  );
  await installAssertledger(setup.repository, setup.home);
  rmSync(setup.runtime);
  const result = Bun.spawnSync([setup.launcher, "--version"], {
    env: { PATH: "/usr/bin:/bin" },
  });
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toBe("assertledger 1.4.0\n");
});

test("rejects incompatible package metadata and unreadable or malformed package inputs", async () => {
  const setup = fixture();
  const packagePath = join(dirname(dirname(setup.cli)), "package.json");
  writeFileSync(
    packagePath,
    JSON.stringify({ version: "1.4.0", engines: { node: "<22.15.0" } }),
  );
  expect(await installationError(setup)).toContain("requires Node <22.15.0");
  writeFileSync(packagePath, "{");
  expect(await installationError(setup)).toContain("SyntaxError");
  writeFileSync(packagePath, "{}");
  expect(await installationError(setup)).toContain("expected string");
  rmSync(packagePath);
  expect(await installationError(setup)).toContain("ENOENT");
  expect(existsSync(setup.launcher)).toBe(false);
});

test("fails explicitly after removal of the installed runtime or package", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  rmSync(setup.runtime);
  const missingRuntime = Bun.spawnSync([setup.launcher, "--version"], {
    env: { PATH: "/usr/bin:/bin" },
  });
  expect(missingRuntime.exitCode).not.toBe(0);
  expect(missingRuntime.stderr.toString()).toContain(
    "No such file or directory",
  );
  symlinkSync(node.executable, setup.runtime);
  rmSync(setup.cli);
  const missingCli = Bun.spawnSync([setup.launcher, "--version"], {
    env: { PATH: "/usr/bin:/bin" },
  });
  expect(missingCli.exitCode).not.toBe(0);
  expect(missingCli.stderr.toString()).toContain("MODULE_NOT_FOUND");
});

test("exec preserves the process identity and forwards interruption", async () => {
  const setup = fixture();
  await installAssertledger(setup.repository, setup.home);
  writeFileSync(
    setup.cli,
    "console.log(process.pid); setInterval(() => {}, 1000);\n",
  );
  const child = Bun.spawn([setup.launcher], {
    env: { PATH: "/usr/bin:/bin" },
    stdout: "pipe",
    stderr: "pipe",
  });
  try {
    const reader = child.stdout.getReader();
    const first = await reader.read();
    expect(Number(new TextDecoder().decode(first.value).trim())).toBe(
      child.pid,
    );
    child.kill("SIGINT");
    await child.exited;
    expect(child.signalCode).toBe("SIGINT");
  } finally {
    child.kill();
    await child.exited;
  }
});
