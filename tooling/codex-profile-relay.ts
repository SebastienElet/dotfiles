import { Socket, createConnection, createServer } from "node:net";
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { z } from "zod";

const maximumMessageBytes = 2048;
const socketTimeoutMilliseconds = 1000;
const maximumConnections = 8;
const acknowledgment = "ok\n";
const messageSchema = z.object({
  systemMessage: z.string().max(maximumMessageBytes),
});

function receiveMessage(
  input: unknown,
  connections: Readonly<Set<Readonly<Socket>>>,
): void {
  const socket = z.instanceof(Socket).parse(input);
  const chunks: Buffer[] = [];
  let size = 0;
  connections.add(socket);
  socket.setTimeout(socketTimeoutMilliseconds, () => {
    socket.destroy();
  });
  socket.on("close", () => {
    connections.delete(socket);
  });
  socket.on("error", () => {
    socket.destroy();
  });
  socket.on("data", (inputData: unknown) => {
    const data = z.instanceof(Buffer).parse(inputData);
    size += data.byteLength;
    if (size > maximumMessageBytes) {
      process.stderr.write(
        "codex-profile: inconnu (diagnostic local trop volumineux)\n",
      );
      socket.destroy();
      return;
    }
    chunks.push(Buffer.from(data));
  });
  socket.on("end", () => {
    try {
      const message = messageSchema.parse(
        JSON.parse(Buffer.concat(chunks).toString("utf8")),
      );
      process.stderr.write(`${message.systemMessage}\n`);
      socket.end(acknowledgment);
    } catch {
      process.stderr.write(
        "codex-profile: inconnu (diagnostic local illisible)\n",
      );
      socket.end();
    }
  });
}

async function startProfileRelay(): Promise<
  Readonly<{ path: string; close: () => Promise<void> }>
> {
  const directory = await mkdtemp("/tmp/codex-profile-");
  const path = join(directory, "observer.sock");
  const connections = new Set<Readonly<Socket>>();
  const server = createServer((socket: unknown) => {
    receiveMessage(socket, connections);
  });
  server.maxConnections = maximumConnections;
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject);
      server.listen(path, resolve);
    });
  } catch (error) {
    await rm(directory, { recursive: true, force: true });
    throw error;
  }
  return {
    path,
    close: async () => {
      for (const socket of connections) {
        socket.destroy();
      }
      try {
        await new Promise<void>((resolve, reject) => {
          server.close((error: Readonly<Error> | undefined) => {
            if (error === undefined) {
              resolve();
            } else {
              reject(error);
            }
          });
        });
      } finally {
        await rm(directory, { recursive: true, force: true });
      }
    },
  };
}

function relayProfileMessage(
  path: string,
  systemMessage: string,
): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = createConnection(path);
    let received = "";
    socket.setTimeout(socketTimeoutMilliseconds, () => {
      socket.destroy();
    });
    socket.on("error", () => {
      socket.destroy();
    });
    socket.on("connect", () => {
      socket.end(JSON.stringify({ systemMessage }));
    });
    socket.on("data", (input: unknown) => {
      received += z.instanceof(Buffer).parse(input).toString("utf8");
      if (received.length > acknowledgment.length) {
        socket.destroy();
      }
    });
    socket.on("close", () => {
      resolve(received === acknowledgment);
    });
  });
}

export { relayProfileMessage, startProfileRelay };
