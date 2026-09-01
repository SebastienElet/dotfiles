type TimeoutGuard = Readonly<{
  clear: () => void;
  expired: () => boolean;
  finish: () => Promise<void>;
}>;

const activeProcessTrees = new Map<number, Set<number>>();
const temporaryPaths = new Set<string>();

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
  let force: ReturnType<typeof setTimeout> | undefined;
  let finish: () => void = () => undefined;
  const finished = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const known = new Set<number>();
  activeProcessTrees.set(child.pid, known);
  const timeout = setTimeout(() => {
    expired = true;
    signalProcessTree(child.pid, "SIGTERM", known);
    force = setTimeout(() => {
      signalProcessTree(child.pid, "SIGKILL", known);
      activeProcessTrees.delete(child.pid);
      finish();
    }, 250);
  }, timeoutMilliseconds);
  return {
    clear: () => {
      clearTimeout(timeout);
      if (!expired) {
        if (force !== undefined) clearTimeout(force);
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
  for (const [pid, known] of active) signalProcessTree(pid, "SIGTERM", known);
  await Bun.sleep(250);
  for (const [pid, known] of active) {
    signalProcessTree(pid, "SIGKILL", known);
    activeProcessTrees.delete(pid);
  }
  await Promise.all(
    [...temporaryPaths].map(async (path) => {
      await rm(path, { force: true, recursive: true });
      temporaryPaths.delete(path);
    }),
  );
}

function signalProcessTree(
  root: number,
  signal: NodeJS.Signals,
  known: Set<number>,
): void {
  for (const pid of descendants(root)) known.add(pid);
  known.add(root);
  for (const pid of [...known].reverse()) {
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
    timeout: 100,
  }).stdout.toString();
  const parents = new Map<number, number[]>();
  for (const line of output.split("\n")) {
    const [pidText, parentText] = line.trim().split(/\s+/u);
    const pid = Number(pidText);
    const parent = Number(parentText);
    if (!Number.isSafeInteger(pid) || !Number.isSafeInteger(parent)) continue;
    parents.set(parent, [...(parents.get(parent) ?? []), pid]);
  }
  const found: number[] = [];
  const pending = [...(parents.get(root) ?? [])];
  while (pending.length > 0) {
    const pid = pending.shift();
    if (pid === undefined) continue;
    found.push(pid);
    pending.push(...(parents.get(pid) ?? []));
  }
  return found;
}

export {
  registerTemporaryPath,
  superviseTimeout,
  terminateEvaluationProcesses,
  unregisterTemporaryPath,
};
import { rm } from "node:fs/promises";
