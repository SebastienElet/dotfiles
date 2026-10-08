import { isAbsolute } from "node:path";
import { z } from "zod";

const environmentSchema = z
  .object({
    HERDR_BIN_PATH: z.string().refine(isAbsolute),
    HERDR_ENV: z.literal("1"),
    HERDR_SOCKET_PATH: z.string().refine(isAbsolute),
  })
  .readonly();
const errorSchema = z.object({
  error: z.object({ code: z.string(), message: z.string() }),
});
const responseSchema = z.object({ result: z.unknown() });
const defaultTimeoutMilliseconds = 35_000;
type NativeEnvironment = z.infer<typeof environmentSchema>;

class HerdrFailureError extends Error {
  public readonly code: string;

  public constructor(code: string, message: string) {
    super(message);
    this.name = "HerdrFailureError";
    this.code = code;
  }
}

function createHerdrCommand(
  environment: Readonly<Record<string, string | undefined>>,
): (
  arguments_: readonly string[],
  options?: Readonly<{ output: "silent" }>,
) => Promise<unknown> {
  const configuration = environmentSchema.parse(environment);
  return async (
    arguments_: readonly string[],
    options?: Readonly<{ output: "silent" }>,
  ): Promise<unknown> => {
    const process = Bun.spawn([configuration.HERDR_BIN_PATH, ...arguments_], {
      env: { ...environment, ...configuration },
      stderr: "pipe",
      stdout: "pipe",
      timeout: defaultTimeoutMilliseconds,
    });
    const [status, stdout, stderr] = await Promise.all([
      process.exited,
      new Response(process.stdout).text(),
      new Response(process.stderr).text(),
    ]);
    if (status !== 0) {
      const parsed = errorSchema.safeParse(parseResponse(stderr));
      if (parsed.success) {
        throw new HerdrFailureError(
          parsed.data.error.code,
          parsed.data.error.message,
        );
      }
      throw new HerdrFailureError(
        "transport_failure",
        stderr.trim() || `Herdr exited ${status}`,
      );
    }
    return options?.output === "silent"
      ? undefined
      : responseSchema.parse(parseResponse(stdout)).result;
  };
}

function parseResponse(value: string): unknown {
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}

export { HerdrFailureError, createHerdrCommand, environmentSchema };
export type { NativeEnvironment };
