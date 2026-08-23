import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { z } from "zod";

const responseSchema = z
  .object({
    jsonrpc: z.literal("2.0"),
    id: z.number().int(),
    result: z.unknown().optional(),
    error: z.unknown().optional(),
  })
  .refine(
    ({ result, error }) => (result === undefined) !== (error === undefined),
  );

const toolsSchema = z.object({
  tools: z.array(z.object({ name: z.string() })),
});

const callResultSchema = z.object({
  content: z.array(z.object({ type: z.string(), text: z.string().optional() })),
  isError: z.boolean().optional(),
});

type PendingRequest = {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};

export class McpClient {
  private server: ChildProcessWithoutNullStreams | undefined;
  private nextId = 0;
  private readonly pending = new Map<number, PendingRequest>();
  private buffer = "";
  private stderr = "";
  private decoder = new TextDecoder("utf-8", { fatal: true });

  constructor(
    private readonly command: readonly [string, ...string[]],
    private readonly repository: string,
    private readonly environment: NodeJS.ProcessEnv,
    private readonly requestTimeoutMilliseconds: number,
    private readonly stopTimeoutMilliseconds: number,
  ) {}

  async start(extraArguments: readonly string[] = []): Promise<void> {
    this.buffer = "";
    this.stderr = "";
    this.decoder = new TextDecoder("utf-8", { fatal: true });
    const [binary, ...prefixArguments] = this.command;
    this.server = spawn(
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
    this.server.stdout.on("data", (chunk: Buffer) => this.consume(chunk));
    this.server.stderr.on("data", (chunk: Buffer) => {
      this.stderr += chunk.toString();
    });
    this.server.once("error", (error) => this.rejectAll(error));
    this.server.once("close", (status, signal) => {
      if (this.pending.size > 0) {
        this.rejectAll(
          new Error(
            `MCP server stopped before replying: status=${status ?? "none"} signal=${signal ?? "none"}\n${this.stderr}`,
          ),
        );
      }
    });

    await this.request("initialize", {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "dotfiles-codegraph-probe", version: "1" },
    });
    this.notify("notifications/initialized");
    const listed = toolsSchema.parse(await this.request("tools/list"));
    const names = listed.tools.map(({ name }) => name).sort();
    if (JSON.stringify(names) !== JSON.stringify(["codegraph_explore"])) {
      throw new Error(`unexpected MCP tools: ${names.join(",")}`);
    }
  }

  async explore(query: string, allowError = false): Promise<string> {
    const result = callResultSchema.parse(
      await this.request("tools/call", {
        name: "codegraph_explore",
        arguments: { query },
      }),
    );
    const text = result.content
      .filter(({ type }) => type === "text")
      .map(({ text }) => text ?? "")
      .join("\n");
    if (result.isError === true && !allowError) {
      throw new Error(`codegraph_explore failed: ${text}`);
    }
    return text;
  }

  async stop(): Promise<void> {
    const server = this.server;
    this.server = undefined;
    if (server === undefined || server.exitCode !== null) return;
    const processId = server.pid;
    const closing = new Promise<void>((resolve) =>
      server.once("close", resolve),
    );
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

  diagnostic(): string {
    return this.stderr;
  }

  private request(method: string, params: unknown = {}): Promise<unknown> {
    const server = this.server;
    if (server === undefined)
      return Promise.reject(new Error("MCP server is not running"));
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`MCP request timed out: ${method}`));
      }, this.requestTimeoutMilliseconds);
      this.pending.set(id, { resolve, reject, timer });
      server.stdin.write(
        `${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`,
      );
    });
  }

  private notify(method: string, params: unknown = {}): void {
    this.server?.stdin.write(
      `${JSON.stringify({ jsonrpc: "2.0", method, params })}\n`,
    );
  }

  private consume(chunk: Buffer): void {
    try {
      this.buffer += this.decoder.decode(chunk, { stream: true });
      for (;;) {
        const newline = this.buffer.indexOf("\n");
        if (newline < 0) return;
        const line = this.buffer.slice(0, newline).trim();
        this.buffer = this.buffer.slice(newline + 1);
        if (!line.startsWith("{")) continue;
        const parsed = responseSchema.safeParse(JSON.parse(line));
        if (!parsed.success) throw new Error("malformed MCP response");
        const waiter = this.pending.get(parsed.data.id);
        if (waiter === undefined) continue;
        this.pending.delete(parsed.data.id);
        clearTimeout(waiter.timer);
        if (parsed.data.error !== undefined) {
          waiter.reject(new Error(JSON.stringify(parsed.data.error)));
        } else {
          waiter.resolve(parsed.data.result);
        }
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

  private rejectAll(error: Error): void {
    for (const waiter of this.pending.values()) {
      clearTimeout(waiter.timer);
      waiter.reject(error);
    }
    this.pending.clear();
  }
}

export function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
