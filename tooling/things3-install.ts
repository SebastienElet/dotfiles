import { join } from "node:path";
import { statSync } from "node:fs";
import { z } from "zod";

const invocationSchema = z.union([
  z.tuple([z.literal("install-cli")]),
  z.tuple([z.literal("verify-app"), z.string().min(1)]),
]);
const environmentSchema = z.object({
  HOME: z.string().min(1),
  SKIP_PAID_APPS: z.string().optional(),
  THINGS3_PAID_CONTEXT: z.literal("1").optional(),
});
const argumentOffset = 2;

function runThingsInstallation(): number {
  const invocation = invocationSchema.parse(Bun.argv.slice(argumentOffset));
  const environment = environmentSchema.parse(process.env);
  if (invocation[0] === "install-cli") {
    if (
      environment.THINGS3_PAID_CONTEXT === "1" &&
      environment.SKIP_PAID_APPS === "1"
    ) {
      return 0;
    }
    const npm = Bun.spawnSync(
      [
        join(environment.HOME, ".volta", "bin", "npm"),
        "install",
        "-g",
        "@dougskinner/thangs",
      ],
      { stdin: "inherit", stdout: "inherit", stderr: "inherit" },
    );
    return npm.exitCode;
  }
  if (environment.SKIP_PAID_APPS === "1") {
    return 0;
  }
  const [, application] = invocation;
  const metadata = statSync(application, { throwIfNoEntry: false });
  if (metadata === undefined || !metadata.isDirectory()) {
    throw new Error(`Homebrew Bundle did not install ${application}`);
  }
  return 0;
}

try {
  process.exitCode = runThingsInstallation();
} catch (error) {
  process.stderr.write(
    `Error: ${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
