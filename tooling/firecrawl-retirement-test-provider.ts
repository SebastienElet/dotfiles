#!/usr/bin/env bun

import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { z } from "zod";

const environmentSchema = z.object({
  FIRECRAWL_RETIREMENT_TEST_LOG: z.string().min(1),
  FIRECRAWL_RETIREMENT_TEST_CLAUDE_CONFIG: z.string().min(1),
  FIRECRAWL_RETIREMENT_TEST_SCENARIO: z.enum([
    "absent",
    "concurrent-configuration",
    "daemon-unavailable",
    "existing",
    "images-only",
    "malformed-docker",
    "persistent-docker",
    "rollback-concurrent-configuration",
  ]),
});
const environment = environmentSchema.parse(process.env);
const cliArgumentStart = 2;
const dockerIdentifierLength = 64;
const usageExitCode = 64;
const command = process.argv.slice(cliArgumentStart);
const program = command[0] === "mcp" ? "codex" : "docker";

appendFileSync(
  environment.FIRECRAWL_RETIREMENT_TEST_LOG,
  `${program} ${command.join(" ")}\n`,
);

function finish(exitCode: number, stdout = "", stderr = ""): never {
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  process.exit(exitCode);
}

if (program === "codex") {
  if (command.join(" ") === "mcp list --json") {
    if (
      environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO ===
      "concurrent-configuration"
    ) {
      writeConcurrentConfiguration();
    }
    finish(
      0,
      ["existing", "rollback-concurrent-configuration"].includes(
        environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO,
      )
        ? '[{"name":"firecrawl","enabled":true},{"name":"unrelated","enabled":true}]\n'
        : '[{"name":"unrelated","enabled":true}]\n',
    );
  }
  if (command.join(" ") === "mcp remove firecrawl") {
    if (
      environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO ===
      "rollback-concurrent-configuration"
    ) {
      writeConcurrentConfiguration();
      finish(1, "", "Codex removal failed\n");
    }
    finish(0);
  }
}

if (program === "docker") {
  const rendered = command.join(" ");
  if (rendered === "info") {
    if (
      environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO === "daemon-unavailable"
    ) {
      finish(1, "", "daemon unavailable\n");
    }
    finish(0, "daemon available\n");
  }
  if (
    rendered ===
    "container ls --all --quiet --filter label=com.docker.compose.project=firecrawl"
  ) {
    if (environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO === "malformed-docker") {
      finish(0, "not-a-container-id\n");
    }
    const scenario = environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO;
    const removed = readFileSync(
      environment.FIRECRAWL_RETIREMENT_TEST_LOG,
      "utf8",
    ).includes("docker container rm ");
    finish(
      0,
      (scenario === "existing" && !removed) || scenario === "persistent-docker"
        ? "aaaaaaaaaaaa\nbbbbbbbbbbbb\n"
        : "",
    );
  }
  if (rendered === "image ls --format {{.Repository}}:{{.Tag}}") {
    finish(
      0,
      ["existing", "images-only"].includes(
        environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO,
      )
        ? "ghcr.io/firecrawl/firecrawl:latest\nghcr.io/firecrawl/playwright-service:latest\nghcr.io/firecrawl/nuq-postgres:latest\nredis:alpine\nrabbitmq:3-management\nunrelated:latest\n"
        : "",
    );
  }
  if (rendered === "container inspect -- aaaaaaaaaaaa bbbbbbbbbbbb") {
    finish(
      0,
      `${JSON.stringify([
        {
          Config: { Image: "ghcr.io/firecrawl/firecrawl:latest" },
          Id: "a".repeat(dockerIdentifierLength),
          Mounts: [
            {
              Name: "anonymous-postgres-volume",
              Type: "volume",
            },
          ],
        },
        {
          Config: { Image: "rabbitmq:3-management" },
          Id: "b".repeat(dockerIdentifierLength),
          Mounts: [],
        },
      ])}\n`,
    );
  }
  if (
    rendered ===
    "volume ls --quiet --filter label=com.docker.compose.project=firecrawl"
  ) {
    const scenario = environment.FIRECRAWL_RETIREMENT_TEST_SCENARIO;
    const removed = readFileSync(
      environment.FIRECRAWL_RETIREMENT_TEST_LOG,
      "utf8",
    ).includes("docker volume rm ");
    finish(
      0,
      (scenario === "existing" && !removed) || scenario === "persistent-docker"
        ? "named-firecrawl-volume\n"
        : "",
    );
  }
  if (rendered.startsWith("container rm ")) {
    finish(0, "aaaaaaaaaaaa\nbbbbbbbbbbbb\n");
  }
  if (rendered === "volume rm -- named-firecrawl-volume") {
    finish(0, "named-firecrawl-volume\n");
  }
}

finish(
  usageExitCode,
  "",
  `unexpected command: ${program} ${command.join(" ")}\n`,
);

function writeConcurrentConfiguration(): void {
  const current = z
    .record(z.string(), z.unknown())
    .parse(
      JSON.parse(
        readFileSync(
          environment.FIRECRAWL_RETIREMENT_TEST_CLAUDE_CONFIG,
          "utf8",
        ),
      ),
    );
  writeFileSync(
    environment.FIRECRAWL_RETIREMENT_TEST_CLAUDE_CONFIG,
    `${JSON.stringify({ ...current, concurrent: true })}\n`,
  );
}
