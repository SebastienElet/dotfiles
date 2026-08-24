type BitbucketIdentity = Readonly<{
  identity: string;
  workspace: string;
  slug: string;
}>;

type RepositoryEvidence = Readonly<{
  identity: string;
  remoteBranch: string;
  uuid: string;
  providerBranch: string;
}>;

const componentPattern = /^[A-Za-z0-9._-]+$/u;
const uuidPattern =
  /^\{[0-9A-Fa-f]{8}(?:-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}\}$/u;
const bitbucketPathComponentCount = 2;

function identityFromPath(path: string): BitbucketIdentity {
  const normalized = path.replace(/^\//u, "").replace(/\.git$/u, "");
  const components = normalized.split("/");
  if (
    components.length !== bitbucketPathComponentCount ||
    components.some((value) => !componentPattern.test(value))
  ) {
    throw new Error("unsupported Bitbucket Cloud fetch URL");
  }
  const [workspace, slug] = components;
  if (workspace === undefined || slug === undefined) {
    throw new Error("unsupported Bitbucket Cloud fetch URL");
  }
  return { identity: `${workspace}/${slug}`, slug, workspace };
}

function parseBitbucketUrl(source: string): URL {
  try {
    return new URL(source);
  } catch {
    throw new Error("unsupported Bitbucket Cloud fetch URL");
  }
}

function parseJson(input: string, errorMessage: string): unknown {
  try {
    return JSON.parse(input);
  } catch {
    throw new Error(errorMessage);
  }
}

function parseBitbucketIdentity(source: string): BitbucketIdentity {
  const scp = /^git@bitbucket\.org:(?<path>.+)$/iu.exec(source);
  const scpPath = scp?.groups?.path;
  if (scpPath !== undefined) {
    return identityFromPath(scpPath);
  }

  const url = parseBitbucketUrl(source);
  const validHttps = url.protocol === "https:";
  const validSsh =
    url.protocol === "ssh:" && url.username === "git" && url.password === "";
  if (
    url.hostname !== "bitbucket.org" ||
    (!validHttps && !validSsh) ||
    url.port !== "" ||
    url.search !== "" ||
    url.hash !== "" ||
    url.pathname.includes("%")
  ) {
    throw new Error("unsupported Bitbucket Cloud fetch URL");
  }
  return identityFromPath(url.pathname);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseCloudContexts(input: string): string {
  const value = parseJson(input, "Bitbucket context list is not valid JSON");
  if (!isRecord(value) || !Array.isArray(value.contexts)) {
    throw new Error("Bitbucket context list has an invalid shape");
  }
  const contexts = value.contexts.filter(
    (context): context is Readonly<{ name: string; host: string }> =>
      isRecord(context) &&
      typeof context.name === "string" &&
      context.name.length > 0 &&
      typeof context.host === "string" &&
      context.host.length > 0,
  );
  if (contexts.length !== value.contexts.length) {
    throw new Error("Bitbucket context list has an invalid shape");
  }
  const cloudContexts = contexts.filter(
    ({ host }) => host === "api.bitbucket.org",
  );
  if (cloudContexts.length !== 1) {
    throw new Error(
      `expected exactly one Bitbucket Cloud context, found ${cloudContexts.length}`,
    );
  }
  const [context] = cloudContexts;
  if (context === undefined) {
    throw new Error("Bitbucket Cloud context is unavailable");
  }
  return context.name;
}

function parseProviderRepository(
  input: string,
  expectedIdentity: string,
): Readonly<{ branch: string; uuid: string }> {
  const value = parseJson(
    input,
    `Bitbucket repository response for ${expectedIdentity} is not valid JSON`,
  );
  if (
    !isRecord(value) ||
    typeof value.uuid !== "string" ||
    !uuidPattern.test(value.uuid) ||
    value.full_name !== expectedIdentity ||
    !isRecord(value.mainbranch) ||
    typeof value.mainbranch.name !== "string" ||
    value.mainbranch.name.length === 0
  ) {
    throw new Error(
      `invalid Bitbucket repository response for ${expectedIdentity}`,
    );
  }
  return {
    branch: value.mainbranch.name,
    uuid: value.uuid,
  } as const;
}

function parseRemoteHead(input: string): string {
  const branches = input
    .split("\n")
    .map(
      (line) =>
        /^ref: refs\/heads\/(?<branch>.+)\s+HEAD$/u.exec(line)?.groups?.branch,
    )
    .filter((branch): branch is string => branch !== undefined);
  if (branches.length === 0) {
    throw new Error("remote did not publish a symbolic HEAD branch");
  }
  if (branches.length !== 1) {
    throw new Error(
      `remote published exactly one symbolic HEAD is required, found ${branches.length}`,
    );
  }
  const [branch] = branches;
  if (branch === undefined) {
    throw new Error("remote symbolic HEAD branch is unavailable");
  }
  return branch;
}

function reconcileProviderEvidence(
  evidence: readonly RepositoryEvidence[],
): string {
  if (evidence.length === 0) {
    throw new Error("no Bitbucket Cloud repository evidence was found");
  }
  const uuids = new Set(evidence.map(({ uuid }) => uuid));
  if (uuids.size !== 1) {
    throw new Error("Bitbucket remotes do not identify the same repository");
  }
  const providerBranches = new Set(
    evidence.map(({ providerBranch }) => providerBranch),
  );
  const remoteBranches = new Set(
    evidence.map(({ remoteBranch }) => remoteBranch),
  );
  if (providerBranches.size !== 1 || remoteBranches.size !== 1) {
    throw new Error("Bitbucket remotes do not agree on one primary branch");
  }
  const [firstEvidence] = evidence;
  if (firstEvidence === undefined) {
    throw new Error("no Bitbucket Cloud repository evidence was found");
  }
  const { providerBranch, remoteBranch } = firstEvidence;
  if (providerBranch !== remoteBranch) {
    throw new Error(
      `provider primary branch ${providerBranch ?? "<missing>"} disagrees with remote HEAD ${remoteBranch ?? "<missing>"}`,
    );
  }
  return providerBranch;
}

export {
  parseBitbucketIdentity,
  parseCloudContexts,
  parseProviderRepository,
  parseRemoteHead,
  reconcileProviderEvidence,
};
export type { BitbucketIdentity, RepositoryEvidence };
