#!/usr/bin/env bun

import { appendFileSync, readFileSync } from "node:fs";
import { z } from "zod";

const environmentSchema = z.object({
  DOCKER_INSTALL_TEST_SCENARIO: z.enum([
    "artifact-absent",
    "artifact-present",
    "command-failure",
    "daemon-unavailable",
    "invalid-evidence",
  ]),
  DOCKER_INSTALL_TEST_STATE: z.string().min(1),
  DOCKER_INSTALL_TEST_TARGET: z.enum([
    "cloakbrowser",
    "firecrawl",
    "scrapling",
  ]),
});
const environment = environmentSchema.parse(process.env);
const cliArgumentStart = 2;
const usageExitCode = 64;
const command = process.argv.slice(cliArgumentStart);
const renderedCommand = command.join(" ");
appendFileSync(environment.DOCKER_INSTALL_TEST_STATE, `${renderedCommand}\n`);

function finish(exitCode: number, stdout = "", stderr = ""): never {
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  process.exit(exitCode);
}

function imageInspectionCount(): number {
  const trace = readFileSync(environment.DOCKER_INSTALL_TEST_STATE, "utf8");
  return trace.split("\n").filter((line) => line.startsWith("image inspect "))
    .length;
}

if (renderedCommand === "info") {
  if (environment.DOCKER_INSTALL_TEST_SCENARIO === "daemon-unavailable") {
    finish(1, "", "daemon unavailable\n");
  }
  finish(0, "test daemon\n");
}

if (command.includes("--help")) {
  finish(0, "Docker help\n");
}

if (command[0] === "compose") {
  if (command.includes("up")) {
    if (environment.DOCKER_INSTALL_TEST_SCENARIO === "command-failure") {
      finish(1, "", "compose up failed\n");
    }
    finish(0);
  }
  if (command.includes("config")) {
    if (environment.DOCKER_INSTALL_TEST_SCENARIO === "invalid-evidence") {
      finish(0, "api\ninvalid service\n");
    }
    finish(0, "api\nplaywright-service\nredis\nrabbitmq\nnuq-postgres\n");
  }
  if (command.includes("ps")) {
    const services =
      environment.DOCKER_INSTALL_TEST_SCENARIO === "artifact-present"
        ? "api\nplaywright-service\nredis\nrabbitmq\nnuq-postgres\n"
        : "api\n";
    finish(0, services);
  }
}

if (command[0] === "image" && command[1] === "inspect") {
  const presentInitially =
    environment.DOCKER_INSTALL_TEST_SCENARIO === "artifact-present";
  const presentAfterInstall =
    environment.DOCKER_INSTALL_TEST_SCENARIO !== "artifact-absent" &&
    imageInspectionCount() > 1;
  finish(presentInitially || presentAfterInstall ? 0 : 1);
}

if (command[0] === "pull") {
  if (environment.DOCKER_INSTALL_TEST_SCENARIO === "command-failure") {
    finish(1, "", "pull failed\n");
  }
  finish(0, `pulled ${command[1] ?? ""}\n`);
}

finish(usageExitCode, "", `unexpected Docker command: ${renderedCommand}\n`);
