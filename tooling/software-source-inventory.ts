import { dirname, isAbsolute, relative, resolve } from "node:path";
import { readFileSync } from "node:fs";
import { z } from "zod";

const composeSchema = z
  .object({
    services: z.record(z.string(), z.looseObject({ image: z.string().min(1) })),
  })
  .loose();
const dockerInstallerInvocationSchema = z
  .object({
    action: z.enum(["install", "verify"]),
    artifact: z.string().min(1),
    policy: z.enum(["allow-skip", "require-docker"]),
    target: z.enum(["cloakbrowser", "firecrawl", "scrapling"]),
  })
  .strict();
const dockerInstallerPattern =
  /^"?(?:[^"\s]*\/)?tooling\/install-docker-artifact"? (?<action>install|verify) (?<target>cloakbrowser|firecrawl|scrapling) "(?<policy>allow-skip|require-docker)" "(?<artifact>[^"\n]+)"$/u;

// Make -n exposes shell text, not expanded argv; this source inventory is advisory.
const channelPatterns = [
  ["channel:homebrew", /\bbrew install\b/u],
  ["channel:mas", /\bmas install\b/u],
  ["channel:npm", /(?:\bnpm install\b|\bvolta install\b)/u],
  ["channel:docker", /(?:\bdocker pull\b|\bdocker run\b|\bdocker compose\b)/u],
  ["channel:fisher", /\bfisher install\b/u],
  ["channel:cargo", /\bcargo install\b/u],
  ["channel:gem", /\bgem install\b/u],
  ["channel:go", /\bgo install\b/u],
  ["channel:npx", /\bnpx\b/u],
  ["channel:pip", /\b(?:pip|pip3|pipx) install\b/u],
  ["channel:uv-tool", /\buv tool install\b/u],
] as const;

function extractRemoteReferences(output: string): readonly string[] {
  const urls = output.match(/https?:\/\/[^\s"'\\)]+/gu) ?? [];
  const gitReferences: string[] = [];
  for (const match of output.matchAll(
    /\bgit clone\s+(?<source>[^\s]+:[^\s]+)(?:\s|$)/gu,
  )) {
    const source = match.groups?.source;
    if (source !== undefined) {
      gitReferences.push(source);
    }
  }
  return [...urls, ...gitReferences];
}

function extractChannels(output: string): readonly string[] {
  const channels: string[] = [];
  for (const [channel, pattern] of channelPatterns) {
    if (pattern.test(output)) {
      channels.push(channel);
    }
  }
  return channels;
}

function extractDockerReferences(output: string): readonly string[] {
  const references: string[] = [];
  for (const match of output.matchAll(
    /\bdocker (?:pull|run)\s+(?<image>[^\s;&|]+)/gu,
  )) {
    const image = match.groups?.image;
    if (image !== undefined) {
      references.push(`docker:${image}`);
    }
  }
  return references;
}

function parseDockerInstallerInvocations(output: string): Readonly<{
  invocations: readonly z.infer<typeof dockerInstallerInvocationSchema>[];
  unsupported: boolean;
}> {
  const installerLines = output
    .split("\n")
    .filter((line) => line.includes("install-docker-artifact"));
  const invocations: z.infer<typeof dockerInstallerInvocationSchema>[] = [];
  let unsupported = false;
  for (const line of installerLines) {
    const match = dockerInstallerPattern.exec(line);
    if (match?.groups === undefined) {
      unsupported = true;
    } else {
      invocations.push(dockerInstallerInvocationSchema.parse(match.groups));
    }
  }
  return { invocations, unsupported };
}

function inventorySources(output: string): readonly string[] {
  const dockerInstaller = parseDockerInstallerInvocations(output);
  const installedDockerArtifacts = dockerInstaller.invocations
    .filter(({ action }) => action === "install")
    .flatMap(({ artifact, target }) =>
      target === "firecrawl"
        ? ["channel:docker"]
        : ["channel:docker", `docker:${artifact}`],
    );
  return [
    ...new Set([
      ...extractChannels(output),
      ...extractDockerReferences(output),
      ...installedDockerArtifacts,
      ...extractRemoteReferences(output),
      ...(dockerInstaller.unsupported
        ? ["docker-installer:unsupported-syntax"]
        : []),
    ]),
  ].toSorted();
}

function extractComposePaths(output: string): readonly string[] {
  const paths: string[] = [];
  for (const match of output.matchAll(
    /\bdocker compose\s+-f\s+(?<path>[^\s;&|]+)/gu,
  )) {
    const path = match.groups?.path;
    if (path !== undefined) {
      paths.push(path);
    }
  }
  for (const { action, artifact, target } of parseDockerInstallerInvocations(
    output,
  ).invocations) {
    if (action === "install" && target === "firecrawl") {
      paths.push(artifact);
    }
  }
  return paths;
}

function usesUnsupportedComposeSyntax(output: string): boolean {
  return output
    .split("\n")
    .flatMap((line) => line.split(/&&|\|\||[;&|]/u))
    .filter((command) => !command.includes("install-docker-artifact"))
    .filter((command) =>
      /\bdocker(?:-compose|\b[^\n]*\bcompose)\b/u.test(command),
    )
    .some((command) => !/\bdocker compose\s+-f\s+[^\s;&|]+/u.test(command));
}

function inventoryComposeSources(
  output: string,
  makefile: string,
): readonly string[] {
  if (usesUnsupportedComposeSyntax(output)) {
    return ["compose:unsupported-syntax"];
  }

  const repositoryRoot = dirname(makefile);
  const sources: string[] = [];
  for (const path of extractComposePaths(output)) {
    const absolutePath = resolve(repositoryRoot, path);
    const repositoryPath = relative(repositoryRoot, absolutePath);
    if (repositoryPath.startsWith("..") || isAbsolute(repositoryPath)) {
      sources.push(`compose:${path}`);
    } else {
      const compose = composeSchema.parse(
        Bun.YAML.parse(readFileSync(absolutePath, "utf8")),
      );
      for (const service of Object.values(compose.services)) {
        sources.push(`docker:${service.image}`);
      }
    }
  }
  return sources;
}

function closingMakeExpression(input: string, contentsStart: number): number {
  let depth = 1;
  for (let index = contentsStart; index < input.length; index += 1) {
    if (input[index] === "$" && input[index + 1] === "(") {
      depth += 1;
      index += 1;
    } else if (input[index] === ")") {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }
  throw new Error("unterminated Make shell expression");
}

function extractParseTimeCommands(makefile: string): readonly string[] {
  const commands: string[] = [];
  const shellExpression = /\$\(\s*shell\s+/gu;
  let match = shellExpression.exec(makefile);
  while (match !== null) {
    const commandStart = shellExpression.lastIndex;
    const commandEnd = closingMakeExpression(makefile, commandStart);
    commands.push(makefile.slice(commandStart, commandEnd));
    shellExpression.lastIndex = commandEnd + 1;
    match = shellExpression.exec(makefile);
  }
  return commands;
}

export { extractParseTimeCommands, inventoryComposeSources, inventorySources };
