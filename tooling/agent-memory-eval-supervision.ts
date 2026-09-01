import { rm } from "node:fs/promises";

type TimeoutGuard = Readonly<{
  clear: () => void;
  expired: () => boolean;
  finish: () => Promise<void>;
}>;

const activeProcessTrees = new Map<number, Set<number>>();
const temporaryPaths = new Set<string>();
const processTerminationGraceMilliseconds = 250;
const processTreeScanTimeoutMilliseconds = 100;

function registerTemporaryPath(path: string): void {
  temporaryPaths.add(path);
}

function unregisterTemporaryPath(path: string): void {
  temporaryPaths.delete(path);
}

function superviseTimeout(
  child: Readonly<{ kill: (signal: NodeJS.Signals) => void; pid: number }>,
  timeoutMilliseconds: number,
): TimeoutGuard {
  let expired = false;
  let force: ReturnType<typeof setTimeout> | null = null;
  let finish: () => void = () => {
    performance.now();
  };
  const finished = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const known = new Set<number>();
  activeProcessTrees.set(child.pid, known);
  const timeout = setTimeout(() => {
    expired = true;
    signalProcessTree(child.pid, "SIGTERM");
    force = setTimeout(() => {
      signalProcessTree(child.pid, "SIGKILL");
      activeProcessTrees.delete(child.pid);
      finish();
    }, processTerminationGraceMilliseconds);
  }, timeoutMilliseconds);
  return {
    clear: () => {
      clearTimeout(timeout);
      if (!expired) {
        if (force !== null) {
          clearTimeout(force);
        }
        activeProcessTrees.delete(child.pid);
        finish();
      }
    },
    expired: () => expired,
    finish: () => finished,
  };
}

async function terminateEvaluationProcesses(): Promise<void> {
  const active = [...activeProcessTrees];
  for (const [pid] of active) {
    signalProcessTree(pid, "SIGTERM");
  }
  await Bun.sleep(processTerminationGraceMilliseconds);
  for (const [pid] of active) {
    signalProcessTree(pid, "SIGKILL");
    activeProcessTrees.delete(pid);
  }
  await Promise.all(
    [...temporaryPaths].map(async (path) => {
      await rm(path, { force: true, recursive: true });
      temporaryPaths.delete(path);
    }),
  );
}

function signalProcessTree(root: number, signal: NodeJS.Signals): void {
  const known = activeProcessTrees.get(root) ?? new Set<number>();
  for (const pid of descendants(root)) {
    known.add(pid);
  }
  known.add(root);
  for (const pid of [...known].toReversed()) {
    try {
      process.kill(pid, signal);
    } catch {
      known.delete(pid);
    }
  }
}

function descendants(root: number): number[] {
  const output = Bun.spawnSync(["/bin/ps", "-axo", "pid=,ppid="], {
    killSignal: "SIGKILL",
    stderr: "ignore",
    stdout: "pipe",
    timeout: processTreeScanTimeoutMilliseconds,
  }).stdout.toString();
  const parents = new Map<number, number[]>();
  for (const line of output.split("\n")) {
    const [pidText, parentText] = line.trim().split(/\s+/u);
    const pid = Number(pidText);
    const parent = Number(parentText);
    if (Number.isSafeInteger(pid) && Number.isSafeInteger(parent)) {
      parents.set(parent, [...(parents.get(parent) ?? []), pid]);
    }
  }
  const found: number[] = [];
  const pending = [...(parents.get(root) ?? [])];
  while (pending.length > 0) {
    const pid = pending.shift();
    if (pid !== undefined) {
      found.push(pid);
      pending.push(...(parents.get(pid) ?? []));
    }
  }
  return found;
}

export {
  registerTemporaryPath,
  superviseTimeout,
  terminateEvaluationProcesses,
  unregisterTemporaryPath,
};
