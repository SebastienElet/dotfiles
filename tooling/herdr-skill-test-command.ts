import { readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const participants = 3;
const barrierTimeoutMilliseconds = 5000;
const hangMilliseconds = 60_000;
const invalidUtf8Byte = 255;
const usageExitCode = 2;
const argumentOffset = 2;
const environment = z
  .object({
    HERDR_SKILL: z.string().default(""),
    HERDR_STATUS: z.coerce.number().int().default(0),
    HERDR_ERROR: z.string().default(""),
    HERDR_MODE: z.enum(["hang", "invalid-utf8"]).optional(),
    HERDR_BARRIER: z.string().optional(),
  })
  .parse(process.env);

if (process.argv.slice(argumentOffset).join(" ") !== "--skill") {
  process.exit(usageExitCode);
}
if (environment.HERDR_BARRIER !== undefined) {
  writeFileSync(join(environment.HERDR_BARRIER, String(process.pid)), "");
  const deadline = Date.now() + barrierTimeoutMilliseconds;
  while (readdirSync(environment.HERDR_BARRIER).length < participants) {
    if (Date.now() > deadline) {
      throw new Error("Herdr test barrier timed out");
    }
    await Bun.sleep(1);
  }
}
if (environment.HERDR_MODE === "hang") {
  await Bun.sleep(hangMilliseconds);
}
if (environment.HERDR_MODE === "invalid-utf8") {
  process.stdout.write(Uint8Array.of(invalidUtf8Byte));
  process.exit(0);
}
process.stdout.write(environment.HERDR_SKILL);
process.stderr.write(environment.HERDR_ERROR);
process.exit(environment.HERDR_STATUS);
