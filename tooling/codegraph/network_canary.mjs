import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import net from "node:net";

const mode = process.argv[2];
const file = fileURLToPath(import.meta.url);

if (mode === "socket") {
  const server = net.createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  await new Promise((resolve) => setTimeout(resolve, 1000));
  await new Promise((resolve) => server.close(resolve));
} else {
  const child = spawn(
    process.execPath,
    [file, mode === "root" ? "child" : "socket"],
    {
      stdio: "ignore",
    },
  );
  const childExit = await new Promise((resolve) => child.once("exit", resolve));
  if (childExit !== 0) process.exitCode = childExit ?? 1;
}
