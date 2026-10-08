import { isAbsolute, relative, sep } from "node:path";
import { realpathSync } from "node:fs";
import { z } from "zod";

const gitTimeoutMilliseconds = 5000;
const repositoryIdentitySchema = z
  .object({
    host: z.string().min(1),
    path: z.string().min(1),
  })
  .readonly();
const repositoryContextSchema = z
  .object({
    branch: z.string().min(1),
    commonDirectory: z.string().refine(isAbsolute),
    gitDirectory: z.string().refine(isAbsolute),
    head: z.string().regex(/^[a-f0-9]{40,64}$/u),
    remotes: z
      .array(
        z
          .object({
            identity: repositoryIdentitySchema.nullable(),
            name: z.string().min(1),
          })
          .readonly(),
      )
      .readonly(),
    root: z.string().refine(isAbsolute),
  })
  .readonly();

type RepositoryContext = z.infer<typeof repositoryContextSchema>;

function inspectRepository(directory: string): RepositoryContext {
  const path = z.string().refine(isAbsolute).parse(directory);
  const root = realpathSync(runGit(path, ["rev-parse", "--show-toplevel"]));
  const relativePath = relative(root, realpathSync(path));
  if (
    relativePath === ".." ||
    relativePath.startsWith(`..${sep}`) ||
    isAbsolute(relativePath)
  ) {
    throw new Error(
      "Git inspection resolved outside the selected repository path",
    );
  }
  const commonDirectory = realpathSync(
    runGit(root, ["rev-parse", "--path-format=absolute", "--git-common-dir"]),
  );
  const gitDirectory = realpathSync(
    runGit(root, ["rev-parse", "--absolute-git-dir"]),
  );
  const branch = runGit(root, ["symbolic-ref", "--quiet", "--short", "HEAD"]);
  const head = runGit(root, ["rev-parse", "HEAD"]);
  const names = runGit(root, ["remote"]);
  const remotes =
    names === ""
      ? []
      : names.split("\n").flatMap((name) =>
          remoteUrls(root, name).map((url) => ({
            identity: remoteIdentity(url),
            name,
          })),
        );
  return repositoryContextSchema.parse({
    branch,
    commonDirectory,
    gitDirectory,
    head,
    remotes,
    root,
  });
}

function runGit(directory: string, arguments_: readonly string[]): string {
  const environment = Object.fromEntries(
    Object.entries(process.env).filter(
      ([name]: readonly [string, string | undefined]) =>
        !name.startsWith("GIT_"),
    ),
  );
  const result = Bun.spawnSync(["git", "-C", directory, ...arguments_], {
    env: { ...environment, GIT_OPTIONAL_LOCKS: "0", GIT_TERMINAL_PROMPT: "0" },
    stderr: "pipe",
    stdout: "pipe",
    timeout: gitTimeoutMilliseconds,
  });
  if (result.signalCode === "SIGTERM") {
    throw new Error("Git inspection timed out or received SIGTERM");
  }
  if (result.exitCode !== 0) {
    const detail = result.stderr.toString().trim();
    throw new Error(
      `Git inspection failed: ${detail === "" ? (result.signalCode ?? result.exitCode) : detail}`,
    );
  }
  return result.stdout.toString().replace(/\n$/u, "");
}

function remoteUrls(directory: string, name: string): readonly string[] {
  const configured = runGit(directory, [
    "config",
    "--null",
    "--get-all",
    `remote.${name}.url`,
  ]);
  if (!configured.endsWith("\0")) {
    throw new Error("Git remote URL configuration has invalid record framing");
  }
  const entries = configured.slice(0, -1).split("\0");
  if (entries.some((url) => /\p{Cc}/u.test(url))) {
    throw new Error("Git remote URL contains control characters");
  }
  const resolved = runGit(directory, [
    "remote",
    "get-url",
    "--all",
    "--",
    name,
  ]).split("\n");
  if (
    resolved.length !== entries.length ||
    resolved.some((url) => /\p{Cc}/u.test(url))
  ) {
    throw new Error(
      "Resolved Git remote URL has ambiguous framing or control characters",
    );
  }
  return resolved;
}

function remoteIdentity(
  value: string,
): z.infer<typeof repositoryIdentitySchema> | null {
  const scp = /^[^:]+:\/\//u.test(value)
    ? null
    : /^(?:[^@:/\s]+@)?(?<host>[^@:/\s]+):(?<path>.+)$/u.exec(value);
  if (scp?.groups?.host !== undefined && scp.groups.path !== undefined) {
    return repositoryIdentitySchema.parse({
      host: scp.groups.host.toLowerCase(),
      path: scp.groups.path.replaceAll(/^\/+|\.git$/gu, ""),
    });
  }
  const parsed = z.url().safeParse(value);
  if (!parsed.success) {
    return null;
  }
  const url = new URL(parsed.data);
  if (!["https:", "ssh:", "git:"].includes(url.protocol)) {
    return null;
  }
  return repositoryIdentitySchema.parse({
    host:
      url.protocol === "ssh:" && url.port === "22" ? url.hostname : url.host,
    path: url.pathname.replaceAll(/^\/+|\.git$/gu, ""),
  });
}

export type { RepositoryContext };
export { inspectRepository };
