import { readFileSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";
import { z } from "zod";

const argumentOffset = 2;
const gateSchema = z
  .tuple([z.enum(["shell", "shell-ci", "fish-syntax", "fish-format"])])
  .rest(z.string().min(1));
const pathsSchema = z.array(z.string().min(1));

function trackedPaths(arguments_: readonly string[]): readonly string[] {
  const result = Bun.spawnSync(["git", ...arguments_], { stderr: "inherit" });
  if (
    arguments_[0] === "grep" &&
    result.exitCode === 1 &&
    result.stdout.length === 0
  ) {
    return [];
  }
  if (result.exitCode !== 0) {
    throw new Error("Git script discovery failed");
  }
  const output = new TextDecoder("utf-8", { fatal: true }).decode(
    result.stdout,
  );
  if (!output.endsWith("\0")) {
    throw new Error("Empty or malformed script discovery");
  }
  return pathsSchema.parse(output.slice(0, -1).split("\0"));
}

function requireCanary(paths: readonly string[], canary: string): void {
  if (!paths.includes(canary)) {
    throw new Error(
      `Script discovery is broken: ${canary} missing from the list`,
    );
  }
}

function discoverShell(): readonly string[] {
  const grep = [
    "grep",
    "-lIz",
    "-E",
    "^#!.*(bash|sh)",
    "--",
    "tooling",
    "install.sh",
  ];
  const candidates = [
    ...trackedPaths(grep),
    ...trackedPaths(["grep", "--cached", ...grep.slice(1)]),
  ];
  const shebang = [...new Set(candidates)].filter((path) =>
    /^#!.*(?:bash|sh)/u.test(
      readFileSync(path, "utf8").split("\n", 1)[0] ?? "",
    ),
  );
  const extension = trackedPaths(["ls-files", "-z", "--", "*.sh"]);
  requireCanary(shebang, "tooling/upgrade");
  requireCanary(extension, "install.sh");
  const paths = [...new Set([...shebang, ...extension])].toSorted();
  for (const path of paths) {
    if (!statSync(path).isFile()) {
      throw new Error(`Expected a Shell file: ${path}`);
    }
  }
  return paths;
}

function selectShellCheck(arguments_: readonly string[]): number {
  const separator = arguments_.indexOf("--");
  if (separator < 1) {
    throw new Error(
      "Expected Shell support paths followed by -- and changed paths",
    );
  }
  const support = new Set(
    arguments_.slice(0, separator).map((path) => workspacePath(path)),
  );
  const changed = arguments_
    .slice(separator + 1)
    .map((path) => workspacePath(path));
  const scripts = new Set(discoverShell());
  const relevant = changed.filter(
    (path) =>
      support.has(path) ||
      scripts.has(path) ||
      path.endsWith(".sh") ||
      path === ".",
  );
  if (changed.length > 0 && relevant.length === 0) {
    process.stdout.write(
      "Shell lint unaffected; no checker preparation required\n",
    );
    return 0;
  }
  const complete = changed.length === 0 || relevant.includes(".");
  const command = complete
    ? ["moon", "run", "repository:shell-lint"]
    : [
        "moon",
        "ci",
        "--stdin",
        "--downstream",
        "none",
        "repository:shell-lint",
      ];
  return Bun.spawnSync(command, {
    stdin: Buffer.from(JSON.stringify({ files: relevant })),
    stdout: "inherit",
    stderr: "inherit",
  }).exitCode;
}

function workspacePath(path: string): string {
  const local = relative(process.cwd(), resolve(path));
  if (local === ".." || local.startsWith("../")) {
    throw new Error(`Changed Shell path is outside the workspace: ${path}`);
  }
  return local || ".";
}

function checkFiles(
  command: readonly string[],
  paths: readonly string[],
): number {
  for (const path of paths) {
    const result = Bun.spawnSync([...command, path], {
      stderr: "inherit",
      stdout: "inherit",
    });
    if (result.exitCode !== 0) {
      return result.exitCode;
    }
  }
  return 0;
}

function main(): number {
  const [gate, ...arguments_] = gateSchema.parse(
    process.argv.slice(argumentOffset),
  );
  if (gate === "shell-ci") {
    return selectShellCheck(arguments_);
  }
  if (arguments_.length > 0) {
    throw new Error(`Unexpected arguments for ${gate}`);
  }
  const commands = {
    shell: ["shellcheck", "--severity=error"],
    "fish-syntax": ["fish", "--no-execute"],
    "fish-format": ["fish_indent", "--check"],
  } as const;
  const command = commands[gate];
  if (Bun.which(command[0]) === null) {
    throw new Error(`${command[0]} is missing: the gate cannot run`);
  }
  const paths =
    gate === "shell"
      ? discoverShell()
      : trackedPaths([
          "ls-files",
          "-z",
          "--",
          "home/.config/fish/*.fish",
          "home/.config/fish/**/*.fish",
        ]);
  if (gate !== "shell") {
    requireCanary(paths, "home/.config/fish/config.fish");
  }
  return checkFiles(command, paths);
}

try {
  process.exitCode = main();
} catch (error) {
  process.stderr.write(
    `${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
