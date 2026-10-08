import type { DispatchOutcome, HerdrPort } from "./dispatch.ts";
import { createHerdrCommand, environmentSchema } from "./herdr-command.ts";
import { dispatchIssue, inspectExistingIssue } from "./dispatch.ts";
import { isAbsolute, join } from "node:path";
import type { IssueSelection } from "./selection.ts";
import { createBindingStore } from "./binding-store.ts";
import { createHash } from "node:crypto";
import { createNativeHerdr } from "./native-herdr.ts";
import { issueNames } from "./prompts.ts";
import { withDispatchLock } from "./dispatch-lock.ts";
import { z } from "zod";

function dispatchSelectedIssue(
  selection: IssueSelection,
): Promise<DispatchOutcome> {
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
  const herdr = createNativeHerdr(
    createHerdrCommand(process.env),
    createBindingStore(directory),
    environmentSchema.parse(process.env),
  );
  return dispatchWithReservation(selection, herdr, directory);
}

async function dispatchWithReservation(
  selection: IssueSelection,
  herdr: HerdrPort,
  directory: string,
): Promise<DispatchOutcome> {
  const existing = await inspectExistingIssue(selection, herdr);
  if (existing !== undefined) {
    return existing;
  }
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

export { dispatchSelectedIssue, dispatchWithReservation };
