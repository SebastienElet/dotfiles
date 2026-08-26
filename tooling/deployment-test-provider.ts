import { appendFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const mode = process.env.DEPLOYMENT_PROVIDER_MODE;
const argumentOffset = 2;
const commandFailureExitCode = 128;
const usageFailureExitCode = 64;

if (mode === "ln") {
  appendFileSync(required("DEPLOYMENT_MARKER"), "called\n");
  forward(required("DEPLOYMENT_REAL_COMMAND"));
} else if (mode === "git") {
  const expected = z
    .array(z.string())
    .parse(JSON.parse(required("DEPLOYMENT_FAIL_ARGUMENTS")));
  if (
    JSON.stringify(process.argv.slice(argumentOffset)) ===
    JSON.stringify(expected)
  ) {
    process.exit(commandFailureExitCode);
  }
  forward(required("DEPLOYMENT_REAL_COMMAND"));
} else if (mode === "fish-success") {
  writeFileSync(
    required("DEPLOYMENT_MARKER"),
    `${process.argv.slice(argumentOffset).join(" ")}\n`,
  );
  const functions = join(required("HOME"), ".config", "fish", "functions");
  mkdirSync(functions, { recursive: true });
  writeFileSync(join(functions, "fzf_configure_bindings.fish"), "");
} else if (mode === "fish-empty") {
  process.exit(0);
} else if (mode === "bun-install") {
  const installArguments = process.argv.slice(argumentOffset);
  const expectedArguments = [
    "--config=/dev/null",
    "--no-env-file",
    "install",
    "--frozen-lockfile",
    "--ignore-scripts",
  ];
  if (JSON.stringify(installArguments) !== JSON.stringify(expectedArguments)) {
    process.exit(commandFailureExitCode);
  }
  appendFileSync(
    required("DEPLOYMENT_MARKER"),
    `${installArguments.join(" ")}\n`,
  );
  const dependency = join(process.cwd(), "node_modules", "zod");
  mkdirSync(dependency, { recursive: true });
  const manifest = join(dependency, "package.json");
  if (!existsSync(manifest)) {
    writeFileSync(manifest, "{}\n");
  }
} else {
  process.stderr.write(
    `unknown deployment provider mode: ${mode ?? "missing"}\n`,
  );
  process.exit(usageFailureExitCode);
}

function forward(command: string): never {
  const result = Bun.spawnSync(
    [command, ...process.argv.slice(argumentOffset)],
    {
      stderr: "inherit",
      stdin: "inherit",
      stdout: "inherit",
    },
  );
  process.exit(result.exitCode);
}

function required(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") {
    throw new Error(`${name} is required`);
  }
  return value;
}
