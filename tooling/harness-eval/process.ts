import { spawn } from "node:child_process";

const MILLISECONDS_PER_SECOND = 1000;
const OUTPUT_LIMIT_BYTES = 4_194_304;
type CaptureOptions = Readonly<{
  cwd: string;
  env: Readonly<Record<string, string | undefined>>;
  stdin: string;
  timeoutSeconds: number;
}>;
type CaptureResult = Readonly<{
  output: string;
  error: "agent-failed" | "timeout" | "output-limit" | null;
}>;

function processGroupStopper(
  child: Readonly<{ pid?: number | undefined }>,
): () => void {
  let signaled = false;
  return (): void => {
    if (child.pid === undefined || signaled) {
      return;
    }
    signaled = true;
    try {
      process.kill(-child.pid, "SIGKILL");
    } catch (error) {
      if (
        !(error instanceof Error && "code" in error && error.code === "ESRCH")
      ) {
        throw error;
      }
    }
  };
}

function capture(
  command: string,
  args: readonly string[],
  options: CaptureOptions,
): Promise<CaptureResult> {
  return new Promise<CaptureResult>((resolve) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      detached: true,
      stdio: ["pipe", "pipe", "ignore"],
    });
    const kill = processGroupStopper(child);
    let output = "";
    let bytes = 0;
    let failure: CaptureResult["error"] = null;
    const timer = setTimeout(() => {
      failure = "timeout";
      kill();
    }, options.timeoutSeconds * MILLISECONDS_PER_SECOND);
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      bytes += Buffer.byteLength(chunk);
      if (bytes > OUTPUT_LIMIT_BYTES) {
        failure = "output-limit";
        kill();
        return;
      }
      output += chunk;
    });
    child.stdin.on("error", () => {
      failure ??= "agent-failed";
    });
    child.on("error", () => {
      failure = "agent-failed";
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      kill();
      resolve({
        output,
        error: failure ?? (code === 0 ? null : "agent-failed"),
      });
    });
    child.stdin.end(options.stdin);
  });
}

export { capture };
export type { CaptureOptions, CaptureResult };
