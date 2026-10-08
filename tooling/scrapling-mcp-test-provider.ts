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
import { dockerImages } from "./docker-image.ts";
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
    imageMismatch: z.boolean().optional(),
    malformedDigest: z.boolean().optional(),
    replaceAfterInspection: z.boolean().optional(),
    imageInspectFailure: z.boolean().optional(),
    invalidImageInspect: z.boolean().optional(),
    differentDigest: z.boolean().optional(),
    delayedOutput: z.boolean().optional(),
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
const sha256HexadecimalLength = 64;
const concurrentListCount = 2;
const concurrentPollMilliseconds = 2;
const invalidByte = 0xff;
const simulatedHangMilliseconds = 60_000;
const dockerCreationFailureExitCode = 125;
const usageFailureExitCode = 64;
const replacedContainerExitCode = 99;
const delayedOutputMilliseconds = 400;
if (process.env.SCRAPLING_TEST_DELAY_CHILD === "1") {
  await Bun.sleep(delayedOutputMilliseconds);
  finish(0);
}
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
const cloak = process.env.CLOAKBROWSER_TEST === "1";
const containerName = cloak ? "cloak" : "scrapling-mcp";
const image =
  process.env[cloak ? "CLOAKBROWSER_IMAGE" : "SCRAPLING_IMAGE"] ??
  dockerImages[cloak ? "cloakbrowser" : "scrapling"];

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
  if (scenario.delayedOutput === true) {
    Bun.spawn([process.execPath, import.meta.path], {
      env: { ...process.env, SCRAPLING_TEST_DELAY_CHILD: "1" },
      stdout: "inherit",
      stderr: "inherit",
    });
  }
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
  finish(0, existsSync(join(state, "container")) ? `${containerName}\n` : "");
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
      Id: "a".repeat(sha256HexadecimalLength),
      Image: `sha256:${(scenario.imageMismatch === true ? "b" : "a").repeat(sha256HexadecimalLength)}`,
      Config: {
        Cmd: cloak ? ["cloakserve", "--idle-timeout=300"] : ["infinity"],
        Entrypoint: cloak ? ["/entrypoint.sh"] : ["sleep"],
        Image: compatible ? image : "other/image",
      },
      HostConfig: {
        ExtraHosts: ["host.docker.internal:host-gateway"],
        PortBindings: {
          "9222/tcp": [
            { HostIp: "127.0.0.1", HostPort: compatible ? "9222" : "9999" },
          ],
        },
      },
      Mounts: [
        {
          Destination: "/profiles",
          Name: "scrapling-profiles",
          RW: true,
          Type: "volume",
        },
      ],
      Name: `/${containerName}`,
      State: { Running: existsSync(join(state, "running")) },
    })}\n`,
  );
}

if (commandArguments[0] === "image" && commandArguments[1] === "inspect") {
  if (scenario.imageInspectFailure === true) {
    finish(1, "", "image inspect failed\n");
  }
  if (scenario.invalidImageInspect === true) {
    finish(0, "not-json\n");
  }
  const repository = image.split("@")[0]?.replace(/:[^/:]+$/u, "");
  const digest =
    scenario.differentDigest === true
      ? `sha256:${"b".repeat(sha256HexadecimalLength)}`
      : image.split("@")[1];
  finish(
    0,
    JSON.stringify({
      Id: `sha256:${"a".repeat(sha256HexadecimalLength)}`,
      RepoDigests: [
        `${repository}@${digest}${scenario.malformedDigest === true ? "@unvalidated" : ""}`,
      ],
    }),
  );
}

if (
  scenario.replaceAfterInspection === true &&
  ["start", "exec"].includes(commandArguments[0] ?? "") &&
  commandArguments[1] !== "a".repeat(sha256HexadecimalLength) &&
  !commandArguments.includes("a".repeat(sha256HexadecimalLength))
) {
  finish(replacedContainerExitCode, "", "replacement container targeted\n");
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
