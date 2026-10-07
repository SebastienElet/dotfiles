import { isAbsolute, join } from "node:path";
import type { DispatchOutcome } from "./dispatch.ts";
import type { IssueSelection } from "./selection.ts";
import { createHash } from "node:crypto";
import { createHerdrCommand } from "./herdr-command.ts";
import { createNativeHerdr } from "./native-herdr.ts";
import { dispatchIssue } from "./dispatch.ts";
import { issueNames } from "./prompts.ts";
import { withDispatchLock } from "./dispatch-lock.ts";
import { z } from "zod";

function dispatchSelectedIssue(
  selection: IssueSelection,
): Promise<DispatchOutcome> {
  const herdr = createNativeHerdr(createHerdrCommand(process.env));
  const environment = z
    .object({
      HERDR_PLUGIN_STATE_DIR: z.string().refine(isAbsolute),
      HERDR_SOCKET_PATH: z.string().refine(isAbsolute),
    })
    .readonly()
    .parse(process.env);
  const sessionKey = createHash("sha256")
    .update(environment.HERDR_SOCKET_PATH)
    .digest("hex");
  const directory = join(
    environment.HERDR_PLUGIN_STATE_DIR,
    "dispatch",
    sessionKey,
  );
  return withDispatchLock(
    directory,
    issueNames(selection).preparation,
    async () => {
      const result = await dispatchIssue(selection, herdr);
      if (result.kind === "uncertain") {
        throw new Error(JSON.stringify(result));
      }
      return result;
    },
  );
}

export { dispatchSelectedIssue };
