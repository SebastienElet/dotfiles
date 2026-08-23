#!/usr/bin/env bun

import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { basename, join } from "node:path";

const arguments_ = process.argv.slice(2);
const realDocker = process.env.SCRAPLING_REAL_DOCKER_BIN;
if (realDocker) {
  if (arguments_[0] === "exec") finish(0, "mcp smoke\n");
  const realArguments = arguments_.map((argument) =>
    argument === "scrapling-profiles:/profiles"
      ? `${process.env.SCRAPLING_REAL_PROFILE_VOLUME}:/profiles`
      : argument,
  );
  if (realArguments[0] === "run") {
    realArguments.splice(
      1,
      0,
      "--label",
      `${process.env.SCRAPLING_REAL_OWNER_LABEL}=${process.env.SCRAPLING_REAL_OWNER}`,
    );
  }
  const result = Bun.spawnSync([realDocker, ...realArguments]);
  if (
    realArguments.includes("container") &&
    realArguments.includes("inspect")
  ) {
    const inspection = result.stdout
      .toString()
      .replaceAll(
        `\"Name\":\"${process.env.SCRAPLING_REAL_PROFILE_VOLUME}\"`,
        '\"Name\":\"scrapling-profiles\"',
      );
    finish(result.exitCode, inspection, result.stderr.toString());
  }
  finish(result.exitCode, result.stdout.toString(), result.stderr.toString());
}

const state = process.env.SCRAPLING_TEST_STATE;
if (!state) throw new Error("SCRAPLING_TEST_STATE is required");
const scenario = JSON.parse(readFileSync(join(state, "scenario.json"), "utf8"));
appendFileSync(join(state, "calls"), `${JSON.stringify(arguments_)}\n`);
const command = arguments_.join(" ");

if (scenario.hang && command.includes(scenario.hang)) {
  await Bun.sleep(60_000);
}

if (scenario.invalidUtf8 && command.includes(scenario.invalidUtf8)) {
  process.stdout.write(new Uint8Array([0xff]));
  process.exit(0);
}

if (arguments_[0] === "info")
  finish(scenario.infoFailure ? 1 : 0, "", "daemon failed\n");

if (command.includes("container ls")) {
  if (scenario.listFailure) finish(1, "", "list failed\n");
  if (scenario.concurrent) {
    writeFileSync(join(state, `list-${process.pid}`), "");
    while (
      readdirSync(state).filter((name) => name.startsWith("list-")).length < 2
    ) {
      Bun.sleepSync(2);
    }
  }
  finish(0, existsSync(join(state, "container")) ? "scrapling-mcp\n" : "");
}

if (command.includes("container inspect")) {
  if (scenario.inspectFailure) finish(1, "", "inspect failed\n");
  if (scenario.invalidInspect) finish(0, "not-json\n");
  const compatible = scenario.compatible !== false;
  finish(
    0,
    `${JSON.stringify({
      Name: "/scrapling-mcp",
      Config: {
        Image: compatible ? "pyd4vinci/scrapling" : "other/image",
        Entrypoint: ["sleep"],
        Cmd: ["infinity"],
      },
      HostConfig: { ExtraHosts: ["host.docker.internal:host-gateway"] },
      Mounts: [
        {
          Type: "volume",
          Name: "scrapling-profiles",
          Destination: "/profiles",
          RW: true,
        },
      ],
      State: { Running: existsSync(join(state, "running")) },
    })}\n`,
  );
}

if (arguments_[0] === "start") {
  if (scenario.startFailure) finish(1, "", "start failed\n");
  writeFileSync(join(state, "running"), "");
  finish(0);
}

if (arguments_[0] === "run") {
  if (scenario.runFailure) finish(125, "", "create failed\n");
  try {
    mkdirSync(join(state, "container"));
    writeFileSync(join(state, "running"), "");
    finish(0);
  } catch {
    finish(125, "", "name conflict\n");
  }
}

if (arguments_[0] === "exec") {
  finish(
    scenario.execExit ?? 0,
    scenario.execStdout ?? "",
    scenario.execStderr ?? "",
  );
}

finish(
  64,
  "",
  `unexpected docker call from ${basename(process.argv[1] ?? "")}: ${command}\n`,
);

function finish(exitCode: number, stdout = "", stderr = ""): never {
  if (stdout) process.stdout.write(stdout);
  if (stderr && exitCode !== 0) process.stderr.write(stderr);
  process.exit(exitCode);
}
