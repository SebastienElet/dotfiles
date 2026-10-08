import type { Bindings } from "./binding-store.ts";
import { isDeepStrictEqual } from "node:util";
import { sessionTokensSchema } from "./sessions.ts";

type BindingRequest =
  | Readonly<{ operation: "read" }>
  | Readonly<{
      operation: "replace";
      pane_id: string;
      expected: Bindings;
      tokens: Readonly<Record<string, string>>;
    }>;
class BindingConflictError extends Error {
  public override readonly name = "BindingConflictError";
}
const retryDelayMilliseconds = 10;
async function patchBinding(
  paneId: string,
  tokens: Readonly<Record<string, string>>,
  request: (data: BindingRequest) => Promise<Bindings>,
): Promise<void> {
  let bindings = await request({ operation: "read" });
  const original = bindings[paneId];
  const updated = { ...original, ...tokens };
  const attempts = 3;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    assertSingleOwner(paneId, updated, bindings);
    try {
      await request({
        operation: "replace",
        pane_id: paneId,
        expected: bindings,
        tokens: updated,
      });
      return;
    } catch (error) {
      if (
        !(error instanceof BindingConflictError) ||
        attempt === attempts - 1
      ) {
        throw error;
      }
      await Bun.sleep(retryDelayMilliseconds);
      bindings = await request({ operation: "read" });
      if (!isDeepStrictEqual(bindings[paneId], original)) {
        throw error;
      }
    }
  }
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
export { BindingConflictError, patchBinding };
export type { BindingRequest };
