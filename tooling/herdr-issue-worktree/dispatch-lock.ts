import { closeSync, mkdirSync, openSync, unlinkSync } from "node:fs";
import { join } from "node:path";

async function withDispatchLock<Result>(
  directory: string,
  key: string,
  action: () => Promise<Result>,
): Promise<Result> {
  mkdirSync(directory, { recursive: true });
  const path = join(directory, `${key}.lock`);
  const descriptor = reserveDispatch(path);
  let completed = false;
  try {
    const result = await action();
    completed = true;
    return result;
  } catch (error) {
    throw new Error(
      `${error instanceof Error ? error.message : String(error)}. Issue dispatch reservation retained at ${path}; inspect native state before manually removing it`,
      { cause: error },
    );
  } finally {
    closeSync(descriptor);
    if (completed) {
      unlinkSync(path);
    }
  }
}

function reserveDispatch(path: string): number {
  try {
    return openSync(path, "wx");
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "EEXIST") {
      throw new Error(
        `Existing issue dispatch lock ${path}. Inspect native panes, agents and Git state before manually removing an abandoned reservation`,
        { cause: error },
      );
    }
    throw error;
  }
}

export { withDispatchLock };
