import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const installer = join(import.meta.dir, "..", "install.sh");
const fixtures: string[] = [];
const executableMode = 0o755;

type Fixture = Readonly<{
  bin: string;
  environment: Readonly<NodeJS.ProcessEnv>;
  home: string;
  root: string;
  trace: string;
}>;

type InstallResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

type GitState =
  | "missing"
  | "system-shim"
  | "unusable"
  | "working"
  | "working-without-clt";

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function createFixture(gitState: GitState): Fixture {
  const root = mkdtempSync(join(tmpdir(), "dotfiles-install-"));
  const bin = join(root, "bin");
  const home = join(root, "home");
  const tracePath = join(root, "trace");
  fixtures.push(root);
  mkdirSync(bin);
  mkdirSync(home);
  installCommand(bin, "uname", String.raw`printf '%s\n' Darwin`);
  installCommand(
    bin,
    "xcode-select",
    `printf 'xcode-select %s\n' "$*" >> "$INSTALL_TEST_TRACE"${gitState === "system-shim" || gitState === "working-without-clt" ? "\nexit 1" : ""}`,
  );
  installCommand(
    bin,
    "make",
    'printf "make %s\\n" "$*" >> "$INSTALL_TEST_TRACE"\nif [ "$1" = moon ]; then exit "$INSTALL_TEST_MOON_STATUS"; fi',
  );
  if (gitState !== "missing") {
    installCommand(
      bin,
      "git",
      `printf 'git %s\n' "$*" >> "$INSTALL_TEST_TRACE"\nif [ "$1" = --version ] && [ ${gitState} = unusable ]; then exit 1; fi\nif [ "$1" = clone ]; then /bin/mkdir -p .dotfiles; fi`,
    );
  }
  const bashEnvironment = join(root, "bash-environment");
  if (gitState === "system-shim") {
    writeFileSync(
      bashEnvironment,
      'command() { if [ "$1" = -v ] && [ "$2" = git ]; then printf \'%s\\n\' /usr/bin/git; else builtin command "$@"; fi; }\n',
    );
  }
  return {
    bin,
    environment: {
      HOME: home,
      INSTALL_TEST_TRACE: tracePath,
      INSTALL_TEST_MOON_STATUS: "0",
      PATH: bin,
      ...(gitState === "system-shim" ? { BASH_ENV: bashEnvironment } : {}),
    },
    home,
    root,
    trace: tracePath,
  };
}

function installCommand(bin: string, name: string, body: string): void {
  const path = join(bin, name);
  writeFileSync(path, `#!/bin/sh\n${body}\n`);
  chmodSync(path, executableMode);
}

async function runInstaller(
  fixture: Fixture,
  input?: string,
): Promise<InstallResult> {
  const child = Bun.spawn(["/bin/bash", installer], {
    env: fixture.environment,
    stderr: "pipe",
    stdin: "pipe",
    stdout: "pipe",
  });
  if (input !== undefined) {
    await child.stdin.write(input);
  }
  await child.stdin.end();
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
}

function readTrace(fixture: Fixture): string {
  try {
    return readFileSync(fixture.trace, "utf8");
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return "";
    }
    throw error;
  }
}

function expectActionableGitFailure(result: InstallResult): void {
  expect(result.exitCode).not.toBe(0);
  expect(`${result.stdout}${result.stderr}`).toContain(
    "xcode-select --install",
  );
  expect(`${result.stdout}${result.stderr}`).toContain(
    "curl -fsSL https://raw.githubusercontent.com/SebastienElet/dotfiles/main/install.sh | bash",
  );
}

test.each([
  ["closed stdin", undefined],
  ["an empty answer", "\n"],
  ["exhausted stdin", "ignored\n"],
  ["a piped acceptance", "y\n"],
])("fails closed with Git absent and %s", async (_description, input) => {
  const fixture = createFixture("missing");
  const result = await runInstaller(fixture, input);

  expectActionableGitFailure(result);
  expect(readTrace(fixture)).toBe("xcode-select --print-path\n");
});

test("fails closed when a Git command is present but unusable", async () => {
  const fixture = createFixture("unusable");
  const result = await runInstaller(fixture);

  expectActionableGitFailure(result);
  expect(readTrace(fixture)).toBe("xcode-select --print-path\ngit --version\n");
});

test("does not invoke the macOS Git shim without developer tools", async () => {
  const fixture = createFixture("system-shim");
  const result = await runInstaller(fixture);

  expectActionableGitFailure(result);
  expect(readTrace(fixture)).toBe("xcode-select --print-path\n");
});

test("rejects a third-party Git when developer tools are absent", async () => {
  const fixture = createFixture("working-without-clt");
  const result = await runInstaller(fixture);

  expectActionableGitFailure(result);
  expect(readTrace(fixture)).toBe("xcode-select --print-path\n");
});

test("bootstraps Moon before installing the workstation", async () => {
  const fixture = createFixture("working");
  const result = await runInstaller(fixture);

  expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  expect(readTrace(fixture)).toBe(
    "xcode-select --print-path\ngit --version\ngit clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles\nmake moon\nmake minimal\n",
  );
});

test("stops before workstation installation when Moon bootstrap fails", async () => {
  const fixture = createFixture("working");
  const bootstrapFailureExitCode = 42;
  const result = await runInstaller({
    ...fixture,
    environment: {
      ...fixture.environment,
      INSTALL_TEST_MOON_STATUS: String(bootstrapFailureExitCode),
    },
  });

  expect(result.exitCode).toBe(bootstrapFailureExitCode);
  expect(readTrace(fixture)).toEndWith("make moon\n");
  expect(readTrace(fixture)).not.toContain("make minimal");
});

test("stops before Moon bootstrap when cloning fails", async () => {
  const fixture = createFixture("working");
  installCommand(
    fixture.bin,
    "git",
    'printf "git %s\\n" "$*" >> "$INSTALL_TEST_TRACE"\nif [ "$1" = clone ]; then exit 1; fi',
  );

  const result = await runInstaller(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(readTrace(fixture)).not.toContain("make ");
});

test("preserves stdin for the Homebrew bootstrap delegated through Moon", async () => {
  const fixture = createFixture("working");
  installCommand(
    fixture.bin,
    "make",
    String.raw`if [ "$1" = minimal ]; then IFS= read -r answer || exit 1; printf "answer %s\n" "$answer" >> "$INSTALL_TEST_TRACE"; fi`,
  );

  const result = await runInstaller(fixture, "y\n");

  expect(result.exitCode).toBe(0);
  expect(readTrace(fixture)).toEndWith("answer y\n");
});
