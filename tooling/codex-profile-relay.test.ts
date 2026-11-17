import { Socket, createServer } from "node:net";
import { expect, test } from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { relayProfileMessage } from "./codex-profile-relay.ts";
import { z } from "zod";

test("bounds delivery when a local relay accepts a connection without responding", async () => {
  const directory = await mkdtemp("/tmp/codex-profile-silent-");
  const path = join(directory, "observer.sock");
  const sockets = new Set<Socket>();
  const server = createServer({ allowHalfOpen: true }, (input: unknown) => {
    const socket = z.instanceof(Socket).parse(input);
    sockets.add(socket);
    socket.resume();
  });
  try {
    await new Promise<void>((resolve, reject) => {
      server.on("error", reject);
      server.listen(path, resolve);
    });
    expect(await relayProfileMessage(path, "codex-profile: inconnu")).toBe(
      false,
    );
  } finally {
    for (const socket of sockets) {
      socket.destroy();
    }
    await new Promise<void>((resolve) => {
      server.close(() => {
        resolve();
      });
    });
    await rm(directory, { recursive: true, force: true });
  }
});
