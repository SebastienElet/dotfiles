#!/usr/bin/env bun
import { readFile, realpath } from "node:fs/promises";
import { diagnosePrerequisites } from "./prerequisites.ts";
import { fetchAgentReachSource } from "./agent-reach-source.ts";
import packageMetadata from "../package.json";
import { prepareAgentReachPlugin } from "./agent-reach-package.ts";
import { z } from "zod";

const sourceSchema = z.strictObject({
  version: z.string().regex(/^\d+\.\d+\.\d+$/u),
  revision: z.string().regex(/^[a-f0-9]{40}$/u),
  python: z.array(z.string().min(1)).min(1).readonly(),
  node: z.array(z.string().min(1)).min(1).readonly(),
});
const invocationSchema = z.union([
  z.tuple([z.literal("plan")]),
  z.tuple([z.literal("doctor")]),
  z.tuple([z.literal("prepare"), z.string().min(1)]),
]);
const argumentOffset = 2;
const indentation = 2;

async function main(): Promise<void> {
  const invocation = invocationSchema.parse(process.argv.slice(argumentOffset));
  const configuration = sourceSchema.parse(
    JSON.parse(
      await readFile(
        new URL("../resources/source.json", import.meta.url),
        "utf8",
      ),
    ),
  );
  if (invocation[0] === "plan") {
    process.stdout.write(
      `${JSON.stringify(configuration, null, indentation)}\n`,
    );
    return;
  }
  if (invocation[0] === "doctor") {
    await diagnosePrerequisites(z.string().min(1).parse(process.env.HOME));
    return;
  }
  if (process.platform !== "darwin" || process.arch !== "arm64") {
    throw new Error("Agent-Reach adaptation targets macOS ARM64");
  }
  const home = await realpath(invocation[1]);
  const source = await fetchAgentReachSource({
    home,
    repository: "https://github.com/Panniantong/Agent-Reach.git",
    revision: configuration.revision,
  });
  const marketplace = await prepareAgentReachPlugin({
    source,
    home,
    version: packageMetadata.version,
    revision: configuration.revision,
  });
  process.stdout.write(`${marketplace}\n`);
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(
      `Agent-Reach: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
