import type {
  MutationWorkflowAdapter,
  MutationWorkflowResult,
  PreparedSnapshot,
} from "./harness-reflection-mutation-workflow-types.ts";

const readMatches = async (
  adapter: MutationWorkflowAdapter,
  path: string,
  expected: string | undefined,
): Promise<boolean> => {
  try {
    return (await adapter.read(path)) === expected;
  } catch {
    return false;
  }
};

const restoreSnapshot = async (
  adapter: MutationWorkflowAdapter,
  snapshot: PreparedSnapshot,
): Promise<boolean> => {
  if (await readMatches(adapter, snapshot.path, snapshot.before)) {
    return true;
  }
  if (!(await readMatches(adapter, snapshot.path, snapshot.contents))) {
    return false;
  }
  try {
    await adapter.replaceMatching(
      snapshot.path,
      snapshot.contents,
      snapshot.before,
    );
  } catch {
    return readMatches(adapter, snapshot.path, snapshot.before);
  }
  return readMatches(adapter, snapshot.path, snapshot.before);
};

const compensate = async (
  adapter: MutationWorkflowAdapter,
  attempted: readonly PreparedSnapshot[],
  events: readonly string[],
): Promise<MutationWorkflowResult> => {
  const unresolvedPaths: string[] = [];
  for (const snapshot of attempted.toReversed()) {
    if (!(await restoreSnapshot(adapter, snapshot))) {
      unresolvedPaths.push(snapshot.path);
    }
  }
  const complete = unresolvedPaths.length === 0;
  return {
    events: [
      ...events,
      "compensation-started",
      complete ? "compensation-completed" : "compensation-incomplete",
    ],
    reason: "apply-or-validation-failed",
    status: complete ? "compensated" : "compensation-incomplete",
    unresolvedPaths,
  };
};

export { compensate, readMatches };
