#!/usr/bin/env bun

import { createServer } from "node:net";
import { spawn } from "node:child_process";

const childExitCode = 1;
const file = import.meta.filename;
const firstArgumentIndex = 2;
const listeningPort = 0;
const [mode] = process.argv.slice(firstArgumentIndex);
const pauseMilliseconds = 1000;

if (mode === "socket") {
  const server = createServer();
  await new Promise<void>((resolve) => {
    server.listen(listeningPort, "127.0.0.1", resolve);
  });
  await Bun.sleep(pauseMilliseconds);
  await new Promise<void>((resolve, reject) => {
    server.close((error?: Readonly<Error>) => {
      if (error) {
        reject(error);
      } else {
        resolve();
      }
    });
  });
} else {
  let childMode = "socket";
  if (mode === "root") {
    childMode = "child";
  }
  const child = spawn(process.execPath, [file, childMode], {
    stdio: "ignore",
  });
  const status = await new Promise<number>((resolve) => {
    child.once("exit", (exitCode) => {
      resolve(exitCode ?? childExitCode);
    });
  });
  if (status !== listeningPort) {
    process.exit(status);
  }
}
