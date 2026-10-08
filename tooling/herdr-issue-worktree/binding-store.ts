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

function createBindingStore(directory: string): BindingStore {
  z.string().refine(isAbsolute).parse(directory);
  const request = async (data: unknown): Promise<Bindings> => {
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
      throw new Error(
        `Issue binding storage failed: ${errors.trim() === "" ? (child.signalCode ?? status) : errors.trim()}`,
      );
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
    patch: async (paneId, tokens) => {
      const bindings = await request({ operation: "read" });
      const updated = { ...bindings[paneId], ...tokens };
      assertSingleOwner(paneId, updated, bindings);
      await request({
        operation: "replace",
        pane_id: paneId,
        expected: bindings,
        tokens: updated,
      });
    },
  };
}

function assertSingleOwner(
  paneId: string,
  tokens: Readonly<Record<string, string>>,
  bindings: Bindings,
): void {
  const identity = sessionTokensSchema.parse(tokens);
  if (
    Object.entries(bindings).some(
      ([candidate, binding]: readonly [
        string,
        Readonly<Record<string, string>>,
      ]) =>
        candidate !== paneId &&
        binding.issue_url === identity.issue_url &&
        binding.issue_repo === identity.issue_repo &&
        binding.issue_role === identity.issue_role,
    )
  ) {
    throw new Error(
      "Another pane already owns this issue role; existing bindings retained",
    );
  }
}

export { createBindingStore };
export type { Bindings, BindingStore };
