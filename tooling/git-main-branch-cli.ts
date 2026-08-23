import {
  parseBitbucketIdentity,
  parseCloudContexts,
  parseProviderRepository,
  parseRemoteHead,
  reconcileProviderEvidence,
  type RepositoryEvidence,
} from "./git-main-branch-core.ts";

type CommandResult = Readonly<{
  status: number;
  stdout: string;
  stderr: string;
}>;

function run(command: string, arguments_: readonly string[]): CommandResult {
  try {
    const result = Bun.spawnSync([command, ...arguments_], {
      stdout: "pipe",
      stderr: "pipe",
    });
    return {
      status: result.exitCode,
      stdout: result.stdout.toString(),
      stderr: result.stderr.toString(),
    };
  } catch (error) {
    return {
      status: 127,
      stdout: "",
      stderr: `${command} is unavailable: ${String(error)}\n`,
    };
  }
}

function fail(message: string, detail = "", status = 1): never {
  if (detail !== "") process.stderr.write(detail);
  process.stderr.write(`git-main-branch: ${message}\n`);
  process.exit(status);
}

function exitWithDetail(detail: string, status: number): never {
  if (detail !== "") process.stderr.write(detail);
  process.exit(status);
}

function validateBranch(
  branch: string,
  gitArguments: readonly string[],
  source: string,
) {
  const result = run("git", [
    ...gitArguments,
    "check-ref-format",
    "--branch",
    branch,
  ]);
  if (result.status !== 0)
    fail(
      `${source} returned an invalid branch ${JSON.stringify(branch)}`,
      result.stderr,
    );
}

function parseArguments(arguments_: readonly string[]) {
  let strict = false;
  let bitbucketCloud = false;
  let offset = 0;
  while (offset < arguments_.length) {
    if (arguments_[offset] === "--strict") strict = true;
    else if (arguments_[offset] === "--bitbucket-cloud") bitbucketCloud = true;
    else break;
    offset += 1;
  }
  return {
    strict,
    bitbucketCloud,
    gitArguments: arguments_.slice(offset),
  } as const;
}

function resolveGeneric(
  strict: boolean,
  gitArguments: readonly string[],
): string {
  const repository = run("git", [...gitArguments, "rev-parse", "--git-dir"]);
  if (repository.status !== 0) {
    if (strict) exitWithDetail(repository.stderr, 1);
    return "main";
  }
  for (const branch of ["main", "master", "trunk"] as const) {
    const result = run("git", [
      ...gitArguments,
      "show-ref",
      "-q",
      "--verify",
      `refs/heads/${branch}`,
    ]);
    if (result.status === 0) return branch;
    if (result.status !== 1) {
      if (strict) exitWithDetail(result.stderr, result.status);
      process.stderr.write(result.stderr);
    }
  }
  return "main";
}

function requireCommand(
  command: string,
  arguments_: readonly string[],
  purpose: string,
): string {
  const result = run(command, arguments_);
  if (result.status !== 0) fail(purpose, result.stderr);
  return result.stdout;
}

function cloudContext(): string {
  const contextOutput = requireCommand(
    "bkt",
    ["context", "list", "--json"],
    "unable to list bkt contexts",
  );
  try {
    return parseCloudContexts(contextOutput);
  } catch (error) {
    fail(error instanceof Error ? error.message : String(error));
  }
}

type ProviderRepository = Readonly<{ uuid: string; branch: string }>;

function providerRepository(
  identity: string,
  context: string,
  gitArguments: readonly string[],
): ProviderRepository {
  const output = requireCommand(
    "bkt",
    ["--context", context, "api", `/repositories/${identity}`, "--json"],
    `unable to query Bitbucket repository ${identity}`,
  );
  let provider: ProviderRepository;
  try {
    provider = parseProviderRepository(output, identity);
  } catch (error) {
    fail(error instanceof Error ? error.message : String(error));
  }
  validateBranch(
    provider.branch,
    gitArguments,
    `Bitbucket repository ${identity}`,
  );
  return provider;
}

function evidenceForRemote(
  remote: string,
  context: string,
  gitArguments: readonly string[],
  providerCache: Map<string, ProviderRepository>,
): RepositoryEvidence[] {
  const urls = requireCommand(
    "git",
    [...gitArguments, "remote", "get-url", "--all", remote],
    `unable to read fetch URLs for remote ${remote}`,
  )
    .split("\n")
    .filter((url) => url !== "");
  if (urls.length === 0) fail(`remote ${remote} has no fetch URL`);

  const headOutput = requireCommand(
    "git",
    [...gitArguments, "ls-remote", "--symref", remote, "HEAD"],
    `unable to read remote HEAD for ${remote}`,
  );
  let remoteBranch: string;
  try {
    remoteBranch = parseRemoteHead(headOutput);
  } catch (error) {
    fail(
      `${remote}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  validateBranch(remoteBranch, gitArguments, `remote ${remote}`);

  return urls.map((url) => {
    let parsed: ReturnType<typeof parseBitbucketIdentity>;
    try {
      parsed = parseBitbucketIdentity(url);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      fail(`remote ${remote}: ${detail}`);
    }
    const provider =
      providerCache.get(parsed.identity) ??
      providerRepository(parsed.identity, context, gitArguments);
    providerCache.set(parsed.identity, provider);
    return {
      identity: parsed.identity,
      remoteBranch,
      uuid: provider.uuid,
      providerBranch: provider.branch,
    };
  });
}

function resolveBitbucketCloud(gitArguments: readonly string[]): string {
  const repository = run("git", [...gitArguments, "rev-parse", "--git-dir"]);
  if (repository.status !== 0)
    fail("unable to inspect the Git repository", repository.stderr);

  const remoteOutput = requireCommand(
    "git",
    [...gitArguments, "remote"],
    "unable to list Git remotes",
  );
  const remotes = remoteOutput.split("\n").filter((remote) => remote !== "");
  if (remotes.length === 0) fail("no Git remote is configured");

  const context = cloudContext();
  const providerCache = new Map<string, ProviderRepository>();
  const evidence = remotes.flatMap((remote) =>
    evidenceForRemote(remote, context, gitArguments, providerCache),
  );
  try {
    return reconcileProviderEvidence(evidence);
  } catch (error) {
    fail(error instanceof Error ? error.message : String(error));
  }
}

export function main(arguments_: readonly string[]): void {
  const { strict, bitbucketCloud, gitArguments } = parseArguments(arguments_);
  const branch = bitbucketCloud
    ? resolveBitbucketCloud(gitArguments)
    : resolveGeneric(strict, gitArguments);
  process.stdout.write(`${branch}\n`);
}
