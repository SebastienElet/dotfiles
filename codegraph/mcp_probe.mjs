import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const repository = fs.realpathSync(process.argv[2]);
const source = path.join(repository, "src");
const privacyEnvironment = {
  ...process.env,
  CODEGRAPH_TELEMETRY: "0",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS: "500",
};
const timings = {};
const auditPauseMilliseconds = Number.parseInt(
  process.env.CODEGRAPH_PROBE_PAUSE_MS || "0",
  10,
);
const serverPidFile = process.env.CODEGRAPH_PROBE_SERVER_PID_FILE;
let server;
let buffer = "";
let nextId = 0;
let pending = new Map();
let stderr = "";
let synchronizationMilliseconds = 0;

const delay = (milliseconds) =>
  new Promise((resolve) => setTimeout(resolve, milliseconds));

const run = (command, args, environment = privacyEnvironment) => {
  const result = spawnSync(command, args, {
    cwd: repository,
    env: environment,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr}`);
  }
  return result.stdout;
};

const request = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++nextId;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`MCP request timed out: ${method}`));
    }, 30000);
    pending.set(id, { resolve, reject, timer });
    server.stdin.write(
      `${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`,
    );
  });

const notify = (method, params = {}) => {
  server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method, params })}\n`);
};

const consume = (chunk) => {
  buffer += chunk.toString();
  for (;;) {
    const newline = buffer.indexOf("\n");
    if (newline < 0) return;
    const line = buffer.slice(0, newline).trim();
    buffer = buffer.slice(newline + 1);
    if (!line.startsWith("{")) continue;
    const message = JSON.parse(line);
    if (message.id === undefined || !pending.has(message.id)) continue;
    const waiter = pending.get(message.id);
    pending.delete(message.id);
    clearTimeout(waiter.timer);
    if (message.error) waiter.reject(new Error(JSON.stringify(message.error)));
    else waiter.resolve(message.result);
  }
};

const startServer = async (extraArguments = []) => {
  buffer = "";
  stderr = "";
  pending = new Map();
  server = spawn(
    "codegraph",
    ["serve", "--mcp", "--path", repository, ...extraArguments],
    {
      cwd: repository,
      env: privacyEnvironment,
      stdio: ["pipe", "pipe", "pipe"],
    },
  );
  if (serverPidFile) fs.writeFileSync(serverPidFile, `${server.pid}\n`);
  server.stdout.on("data", consume);
  server.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  await request("initialize", {
    protocolVersion: "2025-06-18",
    capabilities: {},
    clientInfo: { name: "dotfiles-codegraph-probe", version: "1" },
  });
  notify("notifications/initialized");
  const listed = await request("tools/list");
  const names = (listed.tools || []).map((tool) => tool.name).sort();
  if (JSON.stringify(names) !== JSON.stringify(["codegraph_explore"])) {
    throw new Error(`unexpected MCP tools: ${names.join(",")}`);
  }
};

const stopServer = async () => {
  if (!server) return;
  const closing = new Promise((resolve) => server.once("close", resolve));
  server.stdin.end();
  server.kill("SIGTERM");
  await Promise.race([closing, delay(3000)]);
  server = undefined;
};

const resultText = (result) =>
  (result?.content || [])
    .filter((item) => item?.type === "text")
    .map((item) => item.text || "")
    .join("\n");

const explore = async (query, allowError = false) => {
  const started = Date.now();
  const result = await request("tools/call", {
    name: "codegraph_explore",
    arguments: { query },
  });
  timings[query] = Date.now() - started;
  if (result?.isError && !allowError) {
    throw new Error(`codegraph_explore failed: ${resultText(result)}`);
  }
  return resultText(result);
};

const waitFresh = async (query, current, stale) => {
  let last = "";
  for (let attempt = 0; attempt < 30; attempt += 1) {
    try {
      last = await explore(query);
      if (
        current.every((term) => last.includes(term)) &&
        stale.every((term) => !last.includes(term))
      ) {
        return;
      }
    } catch (error) {
      last = String(error);
    }
    await delay(300);
  }
  throw new Error(`freshness timeout for ${query}: ${last}\n${stderr}`);
};

const replace = (file, before, after) => {
  const current = fs.readFileSync(file, "utf8");
  if (!current.includes(before)) {
    throw new Error(`missing fixture text: ${before}`);
  }
  fs.writeFileSync(file, current.replaceAll(before, after));
};

const branchFile = path.join(source, "branch.ts");
const entryFile = path.join(source, "entry.ts");
const liveFile = path.join(source, "live.ts");
const renamedFile = path.join(source, "renamed-live.ts");
const removableFile = path.join(source, "removable.ts");

try {
  await startServer();
  if (auditPauseMilliseconds > 0) await delay(auditPauseMilliseconds);
  await waitFresh(
    "How does entryValue depend on liveValue, removableValue, and branchMainValue?",
    ["entryValue", "liveValue", "removableValue", "branchMainValue"],
    [],
  );

  run("git", ["switch", "codegraph-alt"]);
  await waitFresh(
    "Where is branchAltValue defined?",
    ["branchAltValue", "FIXTURE_BRANCH_ALT"],
    ["FIXTURE_BRANCH_MAIN"],
  );
  run("git", ["switch", "main"]);
  await waitFresh(
    "Where is branchMainValue defined?",
    ["branchMainValue", "FIXTURE_BRANCH_MAIN"],
    ["branchAltValue", "FIXTURE_BRANCH_ALT"],
  );

  replace(liveFile, "FIXTURE_LIVE_V1", "FIXTURE_LIVE_V2");
  await waitFresh(
    "Where is liveSentinel defined?",
    ["FIXTURE_LIVE_V2"],
    ["FIXTURE_LIVE_V1"],
  );

  fs.renameSync(liveFile, renamedFile);
  replace(renamedFile, "liveValue", "renamedLiveValue");
  replace(entryFile, "./live.js", "./renamed-live.js");
  replace(entryFile, "liveValue", "renamedLiveValue");
  await waitFresh(
    "How does entryValue use renamedLiveValue?",
    ["src/renamed-live.ts", "renamedLiveValue"],
    ["src/live.ts", "liveValue"],
  );

  fs.unlinkSync(removableFile);
  replace(entryFile, 'import { removableValue } from "./removable.js";\n', "");
  replace(entryFile, " + removableValue", "");
  await waitFresh(
    "How is entryValue computed?",
    ["entryValue", "renamedLiveValue"],
    ["removableValue", "FIXTURE_REMOVABLE"],
  );

  await stopServer();
  await startServer();
  await waitFresh(
    "How is entryValue computed after restart?",
    ["entryValue", "renamedLiveValue"],
    ["removableValue", "liveValue"],
  );

  await stopServer();
  await delay(2000);
  await startServer(["--no-watch"]);
  replace(branchFile, "FIXTURE_BRANCH_MAIN", "FIXTURE_WATCHER_INTERRUPTED");
  const immediate = await explore("Where is branchSentinel defined?", true);
  const immediateFresh = immediate.includes("FIXTURE_WATCHER_INTERRUPTED");
  const explicitStale = /stale|out[- ]of[- ]date|sync|refresh|reindex/i.test(
    immediate,
  );
  if (!immediateFresh && !explicitStale) {
    throw new Error(
      `watcher interruption returned silent stale output: ${immediate}`,
    );
  }
  await stopServer();
  const synchronizationStarted = Date.now();
  run("codegraph", ["sync", repository]);
  synchronizationMilliseconds = Date.now() - synchronizationStarted;
  await startServer();
  await waitFresh(
    "Where is branchSentinel defined after reconciliation?",
    ["FIXTURE_WATCHER_INTERRUPTED"],
    ["FIXTURE_BRANCH_MAIN"],
  );
  await stopServer();
  await delay(2000);

  const pidFile = path.join(repository, ".codegraph", "daemon.pid");
  if (fs.existsSync(pidFile)) {
    const record = JSON.parse(fs.readFileSync(pidFile, "utf8"));
    try {
      process.kill(record.pid, 0);
      throw new Error(`CodeGraph daemon still running: ${record.pid}`);
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  }

  process.stdout.write(
    `${JSON.stringify({
      tools: ["codegraph_explore"],
      scenarios: {
        initial: true,
        branchSwitch: true,
        edit: true,
        rename: true,
        delete: true,
        restart: true,
        watcherInterruption: immediateFresh ? "fresh" : "alerted-stale",
        reconciliation: true,
        daemonStopped: true,
      },
      queryLatencyMs: timings,
      synchronizationMs: synchronizationMilliseconds,
    })}\n`,
  );
} finally {
  await stopServer();
}
