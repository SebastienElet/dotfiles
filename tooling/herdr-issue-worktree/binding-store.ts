import { BindingConflictError, patchBinding } from "./binding-patch.ts";
import type { BindingRequest } from "./binding-patch.ts";
import { fileURLToPath } from "node:url";
import { isAbsolute } from "node:path";
import { sessionTokensSchema } from "./sessions.ts";
import { z } from "zod";

const bindingsSchema = z
  .record(z.string(), z.record(z.string(), z.string()).readonly())
  .readonly();
type Bindings = z.infer<typeof bindingsSchema>;
type BindingStore = Readonly<{
  directory: string;
  read: () => Promise<Bindings>;
  patch: (
    paneId: string,
    tokens: Readonly<Record<string, string>>,
  ) => Promise<void>;
}>;
const binary = fileURLToPath(
  new URL("target/release/herdr-issue-state", import.meta.url),
);
const storageTimeoutMilliseconds = 5000;
const unappliedConflictExitCode = 2;

function createBindingStore(directory: string): BindingStore {
  z.string().refine(isAbsolute).parse(directory);
  const request = async (data: BindingRequest): Promise<Bindings> => {
    const child = Bun.spawn([binary, directory], {
      stdin: new Blob([JSON.stringify(data)]),
      stdout: "pipe",
      stderr: "pipe",
      timeout: storageTimeoutMilliseconds,
    });
    const [output, errors, status] = await Promise.all([
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
      child.exited,
    ]);
    if (status !== 0) {
      const message = `Issue binding storage failed: ${errors.trim() === "" ? (child.signalCode ?? status) : errors.trim()}`;
      throw status === unappliedConflictExitCode
        ? new BindingConflictError(message)
        : new Error(message);
    }
    const bindings = bindingsSchema.parse(JSON.parse(output));
    for (const tokens of Object.values(bindings)) {
      sessionTokensSchema.parse(tokens);
    }
    return bindings;
  };
  return {
    directory,
    read: () => request({ operation: "read" }),
    patch: (paneId, tokens) => patchBinding(paneId, tokens, request),
  };
}

export { createBindingStore };
export type { Bindings, BindingStore };
