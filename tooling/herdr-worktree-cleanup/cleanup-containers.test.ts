import { expect, test } from "bun:test";
import type { DockerCommand } from "./cleanup-containers.ts";
import { cleanupWorktreeContainers } from "./cleanup-containers.ts";

const checkoutPath = "/worktrees/issue 492";
const containerIds = ["aaaaaaaaaaaa", "bbbbbbbbbbbb"];

test("stops only IDs returned by the exact Compose path filter", () => {
  const commands: (readonly string[])[] = [];
  const docker: DockerCommand = (arguments_) => {
    commands.push(arguments_);
    return { exitCode: 0, stdout: containerIds.join("\n"), stderr: "" };
  };
  cleanupWorktreeContainers(checkoutPath, "stop", docker);
  expect(commands).toEqual([
    [
      "ps",
      "-q",
      "--filter",
      `label=com.docker.compose.project.working_dir=${checkoutPath}`,
    ],
    ["stop", "--timeout", "10", ...containerIds],
  ]);
});

test("off never invokes Docker", () => {
  const commands: (readonly string[])[] = [];
  cleanupWorktreeContainers(checkoutPath, "off", (arguments_) => {
    commands.push(arguments_);
    return { exitCode: 0, stdout: "", stderr: "" };
  });
  expect(commands).toEqual([]);
});

test("an empty result never invokes stop", () => {
  const commands: (readonly string[])[] = [];
  cleanupWorktreeContainers(checkoutPath, "stop", (arguments_) => {
    commands.push(arguments_);
    return { exitCode: 0, stdout: "", stderr: "" };
  });
  expect(commands).toHaveLength(1);
});

test.each(["missing", "unavailable", "timeout", "invalid", "stop-failed"])(
  "Docker %s cannot fail closure",
  (failure) => {
    const commands: (readonly string[])[] = [];
    const docker: DockerCommand = (arguments_) => {
      commands.push(arguments_);
      if (failure === "missing" || failure === "timeout") {
        throw new Error(failure);
      }
      return {
        exitCode: failure === "unavailable" || arguments_[0] === "stop" ? 1 : 0,
        stdout: failure === "invalid" ? "invalid ID" : containerIds.join("\n"),
        stderr: "",
      };
    };
    expect(() => {
      cleanupWorktreeContainers(checkoutPath, "stop", docker);
    }).not.toThrow();
    expect(commands.map((arguments_) => arguments_[0])).toEqual(
      failure === "stop-failed" ? ["ps", "stop"] : ["ps"],
    );
  },
);
