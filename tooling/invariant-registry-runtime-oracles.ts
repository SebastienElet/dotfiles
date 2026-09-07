import type { InvariantRegistry } from "./invariant-registry-contract.ts";

const runDeclaredOracle = async (
  invariantId: string,
  invocation: readonly string[],
  repositoryRoot: string,
): Promise<void> => {
  let exitCode: number | undefined = undefined;
  try {
    const child = Bun.spawn([...invocation], {
      cwd: repositoryRoot,
      stderr: "ignore",
      stdout: "ignore",
    });
    exitCode = await child.exited;
  } catch {
    throw new Error(`declared oracle failed to start for ${invariantId}`);
  }
  if (exitCode !== 0) {
    throw new Error(`declared oracle failed for ${invariantId}`);
  }
};

const runVerifiedInvariantOracles = async (
  registry: InvariantRegistry,
  repositoryRoot: string,
): Promise<void> => {
  for (const record of registry.invariants) {
    if (
      record.verification.state === "verified" &&
      record.verification.lastRun.oracle !== undefined
    ) {
      await runDeclaredOracle(
        record.id,
        record.verification.lastRun.oracle.invocation,
        repositoryRoot,
      );
    }
  }
};

export { runVerifiedInvariantOracles };
