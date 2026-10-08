#!/usr/bin/env bun

import { appendFileSync } from "node:fs";
import { dockerImages } from "./docker-image.ts";
import { z } from "zod";

const environmentSchema = z.object({
  DOCKER_INSTALL_TEST_SCENARIO: z.enum([
    "artifact-absent",
    "artifact-present",
    "command-failure",
    "daemon-unavailable",
    "invalid-evidence",
    "different-identity",
  ]),
  DOCKER_INSTALL_TEST_STATE: z.string().min(1),
  DOCKER_INSTALL_TEST_TARGET: z.enum(["cloakbrowser", "scrapling"]),
});
const environment = environmentSchema.parse(process.env);
const cliArgumentStart = 2;
const sha256HexadecimalLength = 64;
const image =
  process.env[
    environment.DOCKER_INSTALL_TEST_TARGET === "scrapling"
      ? "SCRAPLING_IMAGE"
      : "CLOAKBROWSER_IMAGE"
  ] ?? dockerImages[environment.DOCKER_INSTALL_TEST_TARGET];
const repository = image.split("@")[0]?.replace(/:[^/:]+$/u, "");
const [, digest] = image.split("@");
const usageExitCode = 64;
const command = process.argv.slice(cliArgumentStart);
const renderedCommand = command.join(" ");
appendFileSync(environment.DOCKER_INSTALL_TEST_STATE, `${renderedCommand}\n`);

function finish(exitCode: number, stdout = "", stderr = ""): never {
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  process.exit(exitCode);
}

if (renderedCommand === "info") {
  if (environment.DOCKER_INSTALL_TEST_SCENARIO === "daemon-unavailable") {
    finish(1, "", "daemon unavailable\n");
  }
  finish(0, "test daemon\n");
}

if (command[0] === "image" && command[1] === "ls") {
  if (environment.DOCKER_INSTALL_TEST_SCENARIO === "invalid-evidence") {
    finish(0, "invalid image evidence\n");
  }
  const output = ["artifact-present", "different-identity"].includes(
    environment.DOCKER_INSTALL_TEST_SCENARIO,
  )
    ? `${JSON.stringify({ Repository: repository, Digest: digest })}\n`
    : "";
  finish(0, output);
}

if (command.includes("--help")) {
  finish(0, "Docker help\n");
}

if (command[0] === "image" && command[1] === "inspect") {
  if (
    !["artifact-present", "different-identity"].includes(
      environment.DOCKER_INSTALL_TEST_SCENARIO,
    )
  ) {
    finish(1, "", "No such image\n");
  }
  const actualDigest =
    environment.DOCKER_INSTALL_TEST_SCENARIO === "different-identity"
      ? `sha256:${"b".repeat(sha256HexadecimalLength)}`
      : digest;
  finish(
    0,
    JSON.stringify({
      Id: `sha256:${"a".repeat(sha256HexadecimalLength)}`,
      RepoDigests: [`${repository}@${actualDigest}`],
    }),
  );
}

if (command[0] === "pull") {
  if (environment.DOCKER_INSTALL_TEST_SCENARIO === "command-failure") {
    finish(1, "", "pull failed\n");
  }
  finish(0, `pulled ${command[1] ?? ""}\n`);
}

finish(usageExitCode, "", `unexpected Docker command: ${renderedCommand}\n`);
