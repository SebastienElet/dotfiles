export type BitbucketIdentity = Readonly<{
  identity: string;
  workspace: string;
  slug: string;
}>;

export type RepositoryEvidence = Readonly<{
  identity: string;
  remoteBranch: string;
  uuid: string;
  providerBranch: string;
}>;

const componentPattern = /^[A-Za-z0-9._-]+$/;
const uuidPattern =
  /^\{[0-9A-Fa-f]{8}(?:-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}\}$/;

function identityFromPath(path: string, source: string): BitbucketIdentity {
  const normalized = path.replace(/^\//, "").replace(/\.git$/, "");
  const components = normalized.split("/");
  if (
    components.length !== 2 ||
    components.some((value) => !componentPattern.test(value))
  ) {
    throw new Error(`unsupported Bitbucket Cloud fetch URL: ${source}`);
  }
  const [workspace, slug] = components;
  if (workspace === undefined || slug === undefined) {
    throw new Error(`unsupported Bitbucket Cloud fetch URL: ${source}`);
  }
  return { identity: `${workspace}/${slug}`, workspace, slug };
}

export function parseBitbucketIdentity(source: string): BitbucketIdentity {
  const scp = /^git@bitbucket\.org:(.+)$/i.exec(source);
  if (scp?.[1]) return identityFromPath(scp[1], source);

  let url: URL;
  try {
    url = new URL(source);
  } catch {
    throw new Error(`unsupported Bitbucket Cloud fetch URL: ${source}`);
  }
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
    throw new Error(`unsupported Bitbucket Cloud fetch URL: ${source}`);
  }
  return identityFromPath(url.pathname, source);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function parseCloudContexts(input: string): string {
  let value: unknown;
  try {
    value = JSON.parse(input);
  } catch {
    throw new Error("Bitbucket context list is not valid JSON");
  }
  if (!isRecord(value) || !Array.isArray(value.contexts))
    throw new Error("Bitbucket context list has an invalid shape");
  const contexts = value.contexts.filter(
    (context): context is { name: string; host: string } =>
      isRecord(context) &&
      typeof context.name === "string" &&
      context.name.length > 0 &&
      typeof context.host === "string" &&
      context.host.length > 0,
  );
  if (contexts.length !== value.contexts.length)
    throw new Error("Bitbucket context list has an invalid shape");
  const cloudContexts = contexts.filter(
    ({ host }) => host === "api.bitbucket.org",
  );
  if (cloudContexts.length !== 1) {
    throw new Error(
      `expected exactly one Bitbucket Cloud context, found ${cloudContexts.length}`,
    );
  }
  const context = cloudContexts[0];
  if (context === undefined)
    throw new Error("Bitbucket Cloud context is unavailable");
  return context.name;
}

export function parseProviderRepository(
  input: string,
  expectedIdentity: string,
) {
  let value: unknown;
  try {
    value = JSON.parse(input);
  } catch {
    throw new Error(
      `Bitbucket repository response for ${expectedIdentity} is not valid JSON`,
    );
  }
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
    uuid: value.uuid,
    branch: value.mainbranch.name,
  } as const;
}

export function parseRemoteHead(input: string): string {
  const branches = input
    .split("\n")
    .map((line) => /^ref: refs\/heads\/(.+)\s+HEAD$/.exec(line)?.[1])
    .filter((branch): branch is string => branch !== undefined);
  if (branches.length === 0)
    throw new Error("remote did not publish a symbolic HEAD branch");
  if (branches.length !== 1)
    throw new Error(
      `remote published exactly one symbolic HEAD is required, found ${branches.length}`,
    );
  const branch = branches[0];
  if (branch === undefined)
    throw new Error("remote symbolic HEAD branch is unavailable");
  return branch;
}

export function reconcileProviderEvidence(
  evidence: readonly RepositoryEvidence[],
): string {
  if (evidence.length === 0)
    throw new Error("no Bitbucket Cloud repository evidence was found");
  const uuids = new Set(evidence.map(({ uuid }) => uuid));
  if (uuids.size !== 1)
    throw new Error("Bitbucket remotes do not identify the same repository");
  const providerBranches = new Set(
    evidence.map(({ providerBranch }) => providerBranch),
  );
  const remoteBranches = new Set(
    evidence.map(({ remoteBranch }) => remoteBranch),
  );
  if (providerBranches.size !== 1 || remoteBranches.size !== 1) {
    throw new Error("Bitbucket remotes do not agree on one primary branch");
  }
  const providerBranch = evidence[0]?.providerBranch;
  const remoteBranch = evidence[0]?.remoteBranch;
  if (providerBranch === undefined || providerBranch !== remoteBranch) {
    throw new Error(
      `provider primary branch ${providerBranch ?? "<missing>"} disagrees with remote HEAD ${remoteBranch ?? "<missing>"}`,
    );
  }
  return providerBranch;
}
