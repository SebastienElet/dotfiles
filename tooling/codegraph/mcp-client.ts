import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { z } from "zod";

const responseSchema = z
  .object({
    error: z.unknown().optional(),
    id: z.number().int(),
    jsonrpc: z.literal("2.0"),
    result: z.unknown().optional(),
  })
  .refine(
    ({ result, error }) => (result === undefined) !== (error === undefined),
  );
const toolsSchema = z.object({
  tools: z.array(z.object({ name: z.string() })),
});
const callResultSchema = z.object({
  content: z.array(z.object({ text: z.string().optional(), type: z.string() })),
  isError: z.boolean().optional(),
});
const byteChunkSchema = z.instanceof(Uint8Array);

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Readonly<Error>) => void;
  timer: ReturnType<typeof setTimeout>;
}

type McpClientOptions = Readonly<{
  command: readonly [string, ...string[]];
  environment: Readonly<NodeJS.ProcessEnv>;
  repository: string;
  requestTimeoutMilliseconds: number;
  stopTimeoutMilliseconds: number;
}>;

class McpClient {
  private server: ChildProcessWithoutNullStreams | undefined;
  private nextId = 0;
  private readonly pending = new Map<number, PendingRequest>();
  private buffer = "";
  private stderr = "";
  private decoder = new TextDecoder("utf-8", { fatal: true });
  private readonly command: readonly [string, ...string[]];
  private readonly environment: Readonly<NodeJS.ProcessEnv>;
  private readonly repository: string;
  private readonly requestTimeoutMilliseconds: number;
  private readonly stopTimeoutMilliseconds: number;

  public constructor(options: McpClientOptions) {
    this.command = options.command;
    this.environment = options.environment;
    this.repository = options.repository;
    this.requestTimeoutMilliseconds = options.requestTimeoutMilliseconds;
    this.stopTimeoutMilliseconds = options.stopTimeoutMilliseconds;
  }

  public async start(extraArguments: readonly string[] = []): Promise<void> {
    this.buffer = "";
    this.stderr = "";
    this.decoder = new TextDecoder("utf-8", { fatal: true });
    const server = this.spawnServer(extraArguments);
    this.server = server;
    this.observeServer();
    await this.request("initialize", {
      capabilities: {},
      clientInfo: { name: "dotfiles-codegraph-probe", version: "1" },
      protocolVersion: "2025-06-18",
    });
    this.notify("notifications/initialized");
    await this.requireExpectedTools();
  }

  private spawnServer(
    extraArguments: readonly string[],
  ): ChildProcessWithoutNullStreams {
    const [binary, ...prefixArguments] = this.command;
    return spawn(
      binary,
      [
        ...prefixArguments,
        "serve",
        "--mcp",
        "--path",
        this.repository,
        ...extraArguments,
      ],
      {
        cwd: this.repository,
        env: this.environment,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
  }

  private observeServer(): void {
    const { server } = this;
    if (server === undefined) {
      throw new Error("MCP server is not running");
    }
    server.stdout.on("data", (chunk: unknown) => {
      this.consume([...byteChunkSchema.parse(chunk)]);
    });
    server.stderr.on("data", (chunk: unknown) => {
      this.stderr += Buffer.from(byteChunkSchema.parse(chunk)).toString();
    });
    server.once("error", (error: unknown) => {
      this.rejectAll(error instanceof Error ? error : new Error(String(error)));
    });
    server.once("close", (status, signal) => {
      if (this.pending.size > 0) {
        this.rejectAll(
          new Error(
            `MCP server stopped before replying: status=${status ?? "none"} signal=${signal ?? "none"}\n${this.stderr}`,
          ),
        );
      }
    });
  }

  private async requireExpectedTools(): Promise<void> {
    const listed = toolsSchema.parse(await this.request("tools/list"));
    const names = listed.tools.map(({ name }) => name).toSorted();
    if (JSON.stringify(names) !== JSON.stringify(["codegraph_explore"])) {
      throw new Error(`unexpected MCP tools: ${names.join(",")}`);
    }
  }

  public async explore(query: string, allowError = false): Promise<string> {
    const result = callResultSchema.parse(
      await this.request("tools/call", {
        arguments: { query },
        name: "codegraph_explore",
      }),
    );
    const text = result.content
      .filter(({ type }) => type === "text")
      .map(({ text: contentText }) => contentText ?? "")
      .join("\n");
    if (result.isError === true && !allowError) {
      throw new Error(`codegraph_explore failed: ${text}`);
    }
    return text;
  }

  public async stop(): Promise<void> {
    const { server } = this;
    this.server = undefined;
    if (server === undefined || server.exitCode !== null) {
      return;
    }
    const processId = server.pid;
    const closing = new Promise<void>((resolve) => {
      server.once("close", resolve);
    });
    server.stdin.end();
    server.kill("SIGTERM");
    const stopped = await Promise.race([
      closing.then(() => true),
      delay(this.stopTimeoutMilliseconds).then(() => false),
    ]);
    if (!stopped) {
      server.kill("SIGKILL");
      throw new Error(`MCP server did not stop: ${processId ?? "unknown"}`);
    }
  }

  public diagnostic(): string {
    return this.stderr;
  }

  private request(method: string, params: unknown = {}): Promise<unknown> {
    const { server } = this;
    if (server === undefined) {
      return Promise.reject(new Error("MCP server is not running"));
    }
    this.nextId += 1;
    const id = this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`MCP request timed out: ${method}`));
      }, this.requestTimeoutMilliseconds);
      this.pending.set(id, { reject, resolve, timer });
      server.stdin.write(
        `${JSON.stringify({ id, jsonrpc: "2.0", method, params })}\n`,
      );
    });
  }

  private notify(method: string, params: unknown = {}): void {
    this.server?.stdin.write(
      `${JSON.stringify({ jsonrpc: "2.0", method, params })}\n`,
    );
  }

  private consume(chunk: readonly number[]): void {
    try {
      this.buffer += this.decoder.decode(Uint8Array.from(chunk), {
        stream: true,
      });
      let consumedLine = this.consumeNextLine();
      while (consumedLine) {
        consumedLine = this.consumeNextLine();
      }
    } catch (error) {
      this.rejectAll(
        new Error(
          `malformed MCP response: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      this.server?.kill("SIGTERM");
    }
  }

  private consumeNextLine(): boolean {
    const newline = this.buffer.indexOf("\n");
    if (newline === -1) {
      return false;
    }
    const line = this.buffer.slice(0, newline).trim();
    this.buffer = this.buffer.slice(newline + 1);
    if (!line.startsWith("{")) {
      return true;
    }
    const parsed = responseSchema.safeParse(JSON.parse(line));
    if (!parsed.success) {
      throw new Error("malformed MCP response");
    }
    const waiter = this.pending.get(parsed.data.id);
    if (waiter === undefined) {
      return true;
    }
    this.pending.delete(parsed.data.id);
    clearTimeout(waiter.timer);
    settleRequest(waiter, parsed.data);
    return true;
  }

  private rejectAll(error: Readonly<Error>): void {
    for (const waiter of this.pending.values()) {
      clearTimeout(waiter.timer);
      waiter.reject(error);
    }
    this.pending.clear();
  }
}

function settleRequest(
  waiter: Readonly<Pick<PendingRequest, "reject" | "resolve">>,
  response: Readonly<{ error?: unknown; result?: unknown }>,
): void {
  if (response.error === undefined) {
    waiter.resolve(response.result);
    return;
  }
  waiter.reject(new Error(JSON.stringify(response.error)));
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });
}

export { delay, McpClient };
