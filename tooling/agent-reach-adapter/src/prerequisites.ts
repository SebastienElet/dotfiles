import { join } from "node:path";
import { runAgentReachCommand } from "./agent-reach-source.ts";

const diagnosticTimeout = 10_000;
const prerequisites: readonly (readonly string[])[] = [
  ["git", "--version"],
  ["python3", "--version"],
  ["uv", "--version"],
  ["docker", "--version"],
  ["ffmpeg", "-version"],
  ["node", "--version"],
  ["mcporter", "--version"],
  ["agent-reach", "--version"],
  ["claude", "--version"],
  ["codex", "--version"],
];

async function diagnosePrerequisites(home: string): Promise<void> {
  for (const args of prerequisites) {
    const [executable] = args;
    if (executable === undefined || Bun.which(executable) === null) {
      throw new Error(`Missing prerequisite: ${executable ?? "command"}`);
    }
    const output = await runAgentReachCommand(args, {}, diagnosticTimeout);
    if (output === "") {
      throw new Error(`Empty prerequisite version: ${executable}`);
    }
    process.stdout.write(`${executable}: ${output.split("\n")[0]}\n`);
  }
  for (const executable of [
    "opencli",
    "twitter",
    "bili",
    "boss",
    "gh",
    "curl",
    "yt-dlp",
    join(home, ".local/share/agent-reach/python-tools/agent-reach/bin/python"),
  ]) {
    if (Bun.which(executable) === null) {
      throw new Error(`Missing prerequisite: ${executable}`);
    }
    process.stdout.write(`${executable}: executable found\n`);
  }
}

export { diagnosePrerequisites };
