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
import { z } from "zod";

const scenarioSchema = z
  .object({
    compatible: z.boolean().optional(),
    concurrent: z.boolean().optional(),
    execExit: z.number().optional(),
    execStderr: z.string().optional(),
    execStdout: z.string().optional(),
    hang: z.string().optional(),
    infoFailure: z.boolean().optional(),
    inspectFailure: z.boolean().optional(),
    invalidInspect: z.boolean().optional(),
    invalidUtf8: z.string().optional(),
    listFailure: z.boolean().optional(),
    present: z.boolean().optional(),
    runFailure: z.boolean().optional(),
    running: z.boolean().optional(),
    startFailure: z.boolean().optional(),
  })
  .strict();

const argumentOffset = 2;
const concurrentListCount = 2;
const concurrentPollMilliseconds = 2;
const invalidByte = 0xff;
const simulatedHangMilliseconds = 60_000;
const dockerCreationFailureExitCode = 125;
const usageFailureExitCode = 64;
const commandArguments = process.argv.slice(argumentOffset);
const realDocker = process.env.SCRAPLING_REAL_DOCKER_BIN;
if (realDocker !== undefined) {
  if (commandArguments[0] === "exec") {
    finish(0, "mcp smoke\n");
  }
  const realArguments = commandArguments.map((argument) =>
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
        `"Name":"${process.env.SCRAPLING_REAL_PROFILE_VOLUME}"`,
        '"Name":"scrapling-profiles"',
      );
    finish(result.exitCode, inspection, result.stderr.toString());
  }
  finish(result.exitCode, result.stdout.toString(), result.stderr.toString());
}

const state = process.env.SCRAPLING_TEST_STATE;
if (state === undefined || state === "") {
  throw new Error("SCRAPLING_TEST_STATE is required");
}
const scenario = scenarioSchema.parse(
  JSON.parse(readFileSync(join(state, "scenario.json"), "utf8")),
);
appendFileSync(join(state, "calls"), `${JSON.stringify(commandArguments)}\n`);
const command = commandArguments.join(" ");

if (scenario.hang !== undefined && command.includes(scenario.hang)) {
  await Bun.sleep(simulatedHangMilliseconds);
}

if (
  scenario.invalidUtf8 !== undefined &&
  command.includes(scenario.invalidUtf8)
) {
  process.stdout.write(new Uint8Array([invalidByte]));
  process.exit(0);
}

if (commandArguments[0] === "info") {
  finish(scenario.infoFailure === true ? 1 : 0, "", "daemon failed\n");
}

if (command.includes("container ls")) {
  if (scenario.listFailure === true) {
    finish(1, "", "list failed\n");
  }
  if (scenario.concurrent === true) {
    writeFileSync(join(state, `list-${process.pid}`), "");
    while (
      readdirSync(state).filter((name) => name.startsWith("list-")).length <
      concurrentListCount
    ) {
      Bun.sleepSync(concurrentPollMilliseconds);
    }
  }
  finish(0, existsSync(join(state, "container")) ? "scrapling-mcp\n" : "");
}

if (command.includes("container inspect")) {
  if (scenario.inspectFailure === true) {
    finish(1, "", "inspect failed\n");
  }
  if (scenario.invalidInspect === true) {
    finish(0, "not-json\n");
  }
  const compatible = scenario.compatible !== false;
  finish(
    0,
    `${JSON.stringify({
      Config: {
        Cmd: ["infinity"],
        Entrypoint: ["sleep"],
        Image: compatible ? "pyd4vinci/scrapling" : "other/image",
      },
      HostConfig: { ExtraHosts: ["host.docker.internal:host-gateway"] },
      Mounts: [
        {
          Destination: "/profiles",
          Name: "scrapling-profiles",
          RW: true,
          Type: "volume",
        },
      ],
      Name: "/scrapling-mcp",
      State: { Running: existsSync(join(state, "running")) },
    })}\n`,
  );
}

if (commandArguments[0] === "start") {
  if (scenario.startFailure === true) {
    finish(1, "", "start failed\n");
  }
  writeFileSync(join(state, "running"), "");
  finish(0);
}

if (commandArguments[0] === "run") {
  if (scenario.runFailure === true) {
    finish(dockerCreationFailureExitCode, "", "create failed\n");
  }
  try {
    mkdirSync(join(state, "container"));
    writeFileSync(join(state, "running"), "");
    finish(0);
  } catch {
    finish(dockerCreationFailureExitCode, "", "name conflict\n");
  }
}

if (commandArguments[0] === "exec") {
  finish(
    scenario.execExit ?? 0,
    scenario.execStdout ?? "",
    scenario.execStderr ?? "",
  );
}

finish(
  usageFailureExitCode,
  "",
  `unexpected docker call from ${basename(process.argv[1] ?? "")}: ${command}\n`,
);

function finish(exitCode: number, stdout = "", stderr = ""): never {
  if (stdout) {
    process.stdout.write(stdout);
  }
  if (stderr && exitCode !== 0) {
    process.stderr.write(stderr);
  }
  process.exit(exitCode);
}
