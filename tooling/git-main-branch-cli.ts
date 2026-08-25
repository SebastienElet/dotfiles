import {
  type RepositoryEvidence,
  parseBitbucketIdentity,
  parseCloudContexts,
  parseProviderRepository,
  parseRemoteHead,
  reconcileProviderEvidence,
} from "./git-main-branch-core.ts";

type CommandResult = Readonly<{
  status: number;
  stdout: string;
  stderr: string;
}>;

function run(command: string, arguments_: readonly string[]): CommandResult {
  try {
    const result = Bun.spawnSync([command, ...arguments_], {
      stderr: "pipe",
      stdout: "pipe",
    });
    return {
      status: result.exitCode,
      stderr: result.stderr.toString(),
      stdout: result.stdout.toString(),
    };
  } catch (error) {
    return {
      status: 127,
      stderr: `${command} is unavailable: ${String(error)}\n`,
      stdout: "",
    };
  }
}

function fail(message: string, detail = "", status = 1): never {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.stderr.write(`git-main-branch: ${message}\n`);
  process.exit(status);
}

function exitWithDetail(detail: string, status: number): never {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.exit(status);
}

function validateBranch(
  branch: string,
  gitArguments: readonly string[],
  source: string,
): void {
  const result = run("git", [
    ...gitArguments,
    "check-ref-format",
    "--branch",
    branch,
  ]);
  if (result.status !== 0) {
    fail(
      `${source} returned an invalid branch ${JSON.stringify(branch)}`,
      result.stderr,
    );
  }
}

function parseArguments(arguments_: readonly string[]): Readonly<{
  bitbucketCloud: boolean;
  gitArguments: readonly string[];
  strict: boolean;
}> {
  let bitbucketCloud = false;
  let offset = 0;
  let strict = false;
  while (offset < arguments_.length) {
    if (arguments_[offset] === "--strict") {
      strict = true;
    } else if (arguments_[offset] === "--bitbucket-cloud") {
      bitbucketCloud = true;
    } else {
      break;
    }
    offset += 1;
  }
  return {
    bitbucketCloud,
    gitArguments: arguments_.slice(offset),
    strict,
  } as const;
}

function resolveGeneric(
  strict: boolean,
  gitArguments: readonly string[],
): string {
  const repository = run("git", [...gitArguments, "rev-parse", "--git-dir"]);
  if (repository.status !== 0) {
    if (strict) {
      exitWithDetail(repository.stderr, 1);
    }
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
    if (result.status === 0) {
      return branch;
    }
    if (result.status !== 1) {
      if (strict) {
        exitWithDetail(result.stderr, result.status);
      }
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
  if (result.status !== 0) {
    fail(purpose, result.stderr);
  }
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
    return fail(error instanceof Error ? error.message : String(error));
  }
}

type ProviderRepository = Readonly<{ uuid: string; branch: string }>;
type RemoteEvidenceOptions = Readonly<{
  gitArguments: readonly string[];
  providerForIdentity: (identity: string) => ProviderRepository;
}>;

function parseProviderEvidence(
  output: string,
  identity: string,
): ProviderRepository {
  try {
    return parseProviderRepository(output, identity);
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
}

function remoteBranchFor(
  remote: string,
  output: string,
  gitArguments: readonly string[],
): string {
  try {
    const branch = parseRemoteHead(output);
    validateBranch(branch, gitArguments, `remote ${remote}`);
    return branch;
  } catch (error) {
    return fail(
      `${remote}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function identityForRemote(remote: string, url: string): string {
  try {
    return parseBitbucketIdentity(url).identity;
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return fail(`remote ${remote}: ${detail}`);
  }
}

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
  const provider = parseProviderEvidence(output, identity);
  validateBranch(
    provider.branch,
    gitArguments,
    `Bitbucket repository ${identity}`,
  );
  return provider;
}

function evidenceForRemote(
  remote: string,
  options: RemoteEvidenceOptions,
): RepositoryEvidence[] {
  const { gitArguments, providerForIdentity } = options;
  const urls = requireCommand(
    "git",
    [...gitArguments, "remote", "get-url", "--all", remote],
    `unable to read fetch URLs for remote ${remote}`,
  )
    .split("\n")
    .filter((url) => url !== "");
  if (urls.length === 0) {
    fail(`remote ${remote} has no fetch URL`);
  }

  const headOutput = requireCommand(
    "git",
    [...gitArguments, "ls-remote", "--symref", remote, "HEAD"],
    `unable to read remote HEAD for ${remote}`,
  );
  const remoteBranch = remoteBranchFor(remote, headOutput, gitArguments);

  return urls.map((url) => {
    const identity = identityForRemote(remote, url);
    const provider = providerForIdentity(identity);
    return {
      identity,
      providerBranch: provider.branch,
      remoteBranch,
      uuid: provider.uuid,
    };
  });
}

function resolveBitbucketCloud(gitArguments: readonly string[]): string {
  const repository = run("git", [...gitArguments, "rev-parse", "--git-dir"]);
  if (repository.status !== 0) {
    fail("unable to inspect the Git repository", repository.stderr);
  }

  const remoteOutput = requireCommand(
    "git",
    [...gitArguments, "remote"],
    "unable to list Git remotes",
  );
  const remotes = remoteOutput.split("\n").filter((remote) => remote !== "");
  if (remotes.length === 0) {
    fail("no Git remote is configured");
  }

  const context = cloudContext();
  const providerCache = new Map<string, ProviderRepository>();
  const providerForIdentity = (identity: string): ProviderRepository => {
    const cached = providerCache.get(identity);
    if (cached !== undefined) {
      return cached;
    }
    const provider = providerRepository(identity, context, gitArguments);
    providerCache.set(identity, provider);
    return provider;
  };
  const evidence = remotes.flatMap((remote) =>
    evidenceForRemote(remote, { gitArguments, providerForIdentity }),
  );
  try {
    return reconcileProviderEvidence(evidence);
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
}

export function main(arguments_: readonly string[]): void {
  const { strict, bitbucketCloud, gitArguments } = parseArguments(arguments_);
  const branch = bitbucketCloud
    ? resolveBitbucketCloud(gitArguments)
    : resolveGeneric(strict, gitArguments);
  process.stdout.write(`${branch}\n`);
}
