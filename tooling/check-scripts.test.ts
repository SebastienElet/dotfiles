import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const roots: string[] = [];
const executableMode = 0o755;
const failureStatus = 31;
const entrypoint = join(import.meta.dir, "check-scripts.ts");
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function fixture(): string {
  const root = mkdtempSync(join(tmpdir(), "check-scripts-"));
  roots.push(root);
  mkdirSync(join(root, "tooling"));
  mkdirSync(join(root, "bin"));
  mkdirSync(join(root, "home/.config/fish"), { recursive: true });
  writeFileSync(join(root, "tooling/upgrade"), "#!/bin/bash\ntrue\n");
  writeFileSync(join(root, "install.sh"), "true\n");
  writeFileSync(join(root, "home/.config/fish/config.fish"), "true\n");
  for (const command of ["shellcheck", "fish", "fish_indent"]) {
    const executable = join(root, "bin", command);
    writeFileSync(
      executable,
      '#!/bin/sh\nfor file in "$@"; do case "$file" in *broken*) echo rejected >&2; exit 31;; esac; done\n',
    );
    chmodSync(executable, executableMode);
  }
  expect(Bun.spawnSync(["git", "init", "-q", root]).exitCode).toBe(0);
  index(root);
  return root;
}

function index(root: string): void {
  expect(Bun.spawnSync(["git", "add", "."], { cwd: root }).exitCode).toBe(0);
}

function run(root: string, gate = "shell"): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync([process.execPath, entrypoint, gate], {
    cwd: root,
    env: {
      ...process.env,
      PATH: `${join(root, "bin")}:${process.env.PATH ?? ""}`,
    },
  });
}

test("accepts tracked scripts when their checker succeeds", () => {
  expect(run(fixture()).exitCode).toBe(0);
});

test.each(["tooling/broken", "broken name.sh"])(
  "checks newly indexed %s and propagates tool failure",
  (path) => {
    const root = fixture();
    writeFileSync(join(root, path), "#!/bin/bash\ntrue\n");
    index(root);
    const result = run(root);
    expect(result.exitCode).toBe(failureStatus);
    expect(result.stderr.toString()).toContain("rejected");
  },
);

test.each(["tooling/upgrade", "install.sh"])(
  "refuses discovery without %s",
  (path) => {
    const root = fixture();
    rmSync(join(root, path));
    index(root);
    const result = run(root);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain("discovery");
  },
);

test("refuses a Git discovery failure", () => {
  const root = fixture();
  rmSync(join(root, ".git"), { recursive: true });
  expect(run(root).exitCode).not.toBe(0);
});

test.each(["fish-syntax", "fish-format"])(
  "%s checks a tracked Fish file with spaces",
  (gate) => {
    const root = fixture();
    writeFileSync(join(root, "home/.config/fish/broken name.fish"), "broken\n");
    index(root);
    expect(run(root, gate).exitCode).toBe(failureStatus);
  },
);

test("refuses a missing checker", () => {
  const root = fixture();
  for (const command of ["shellcheck", "fish", "fish_indent"]) {
    rmSync(join(root, "bin", command));
  }
  const result = Bun.spawnSync([process.execPath, entrypoint, "shell"], {
    cwd: root,
    env: { PATH: join(root, "bin") },
  });
  expect(result.exitCode).not.toBe(0);
});

test("refuses malformed discovery output", () => {
  const root = fixture();
  const git = join(root, "bin/git");
  writeFileSync(git, "#!/bin/sh\nprintf malformed\n");
  chmodSync(git, executableMode);
  const result = run(root);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("malformed script discovery");
});
