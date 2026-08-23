#!/usr/bin/env bun

import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { createServer } from "node:net";

const mode = process.argv[2];
const file = fileURLToPath(import.meta.url);

if (mode === "socket") {
  const server = createServer();
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  await Bun.sleep(1_000);
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error === undefined ? resolve() : reject(error))),
  );
} else {
  const child = spawn(
    process.execPath,
    [file, mode === "root" ? "child" : "socket"],
    {
      stdio: "ignore",
    },
  );
  const status = await new Promise<number | null>((resolve) =>
    child.once("exit", resolve),
  );
  if (status !== 0) process.exit(status ?? 1);
}
