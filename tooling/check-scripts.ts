import { z } from "zod";

const argumentOffset = 2;
const gateSchema = z.tuple([z.enum(["shell", "fish-syntax", "fish-format"])]);
const pathsSchema = z.array(z.string().min(1));

function trackedPaths(arguments_: readonly string[]): readonly string[] {
  const result = Bun.spawnSync(["git", ...arguments_], { stderr: "inherit" });
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
  const shebang = trackedPaths([
    "grep",
    "-lIz",
    "-E",
    "^#!.*(bash|sh)",
    "--",
    "tooling",
    "install.sh",
  ]);
  const extension = trackedPaths(["ls-files", "-z", "--", "*.sh"]);
  requireCanary(shebang, "tooling/upgrade");
  requireCanary(extension, "install.sh");
  return [...new Set([...shebang, ...extension])].toSorted();
}

function main(): number {
  const [gate] = gateSchema.parse(process.argv.slice(argumentOffset));
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

try {
  process.exitCode = main();
} catch (error) {
  process.stderr.write(
    `${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
