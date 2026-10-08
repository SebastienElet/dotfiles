import { createHerdrCommand, environmentSchema } from "./herdr-command.ts";
import { createBindingStore } from "./binding-store.ts";
import { createNativeHerdr } from "./native-herdr.ts";
import { isAbsolute } from "node:path";
import { registerIssueWork } from "./work-binding.ts";
import { z } from "zod";

if (import.meta.main) {
  const directory = z.string().refine(isAbsolute).parse(process.argv[2]);
  try {
    const nativeEnvironment = environmentSchema.parse({
      ...process.env,
      HERDR_SOCKET_PATH: process.argv[3],
      HERDR_BIN_PATH: process.argv[4],
    });
    await registerIssueWork(
      await Bun.stdin.text(),
      createNativeHerdr(
        createHerdrCommand({ ...process.env, ...nativeEnvironment }),
        createBindingStore(directory),
        nativeEnvironment,
      ),
    );
    process.stdout.write("Issue worktree binding registered\n");
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { registerIssueWork } from "./work-binding.ts";
