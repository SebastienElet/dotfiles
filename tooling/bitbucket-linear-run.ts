import type { Config, PullRequest, Team } from "./bitbucket-linear-schema.ts";
import {
  type IssuePlan,
  attachmentsMatch,
  correlate,
  planIssue,
  sameIssueState,
} from "./bitbucket-linear-core.ts";
import type { Providers } from "./bitbucket-linear-providers.ts";

type Observation = Readonly<{
  status:
    | "unchanged"
    | "linked"
    | "updated"
    | "completed"
    | "ambiguous"
    | "ignored"
    | "failed";
  identifier?: string;
  repository?: string;
  url?: string;
  reason?: string;
  observedState?: string;
}>;
type Plan = Readonly<{
  identity: Awaited<ReturnType<Providers["identity"]>>;
  issues: readonly IssuePlan[];
  observations: readonly Observation[];
}>;
type Result = Readonly<{
  plan: Plan;
  observations: readonly Observation[];
  exitCode: 0 | 1;
}>;
type RunOptions = Readonly<{
  apply: boolean;
  announce: (plan: Plan) => unknown;
}>;
type Inventory = Readonly<{
  teams: Readonly<Record<string, Team>>;
  blocked: readonly string[];
  prs: readonly PullRequest[];
  observations: readonly Observation[];
}>;
type Associations = Readonly<{
  matches: Readonly<Record<string, readonly PullRequest[]>>;
  blocked: readonly string[];
  observations: readonly Observation[];
}>;
async function inventory(
  config: Config,
  providers: Providers,
): Promise<Inventory> {
  const teams = new Map<string, Team>();
  const blocked = new Set<string>();
  const prs: PullRequest[] = [];
  const observations: Observation[] = [];
  for (const repo of config.repositories) {
    try {
      if (!teams.has(repo.teamKey)) {
        teams.set(repo.teamKey, await providers.team(repo));
      }
      prs.push(...(await providers.pullRequests(repo)));
    } catch {
      blocked.add(repo.teamKey);
      observations.push({
        status: "failed",
        repository: `${repo.workspace}/${repo.repository}`,
        reason: "inventory-or-team-failed",
      });
    }
  }
  return {
    teams: Object.fromEntries(teams),
    blocked: [...blocked],
    prs,
    observations,
  };
}
function associate(
  prs: readonly PullRequest[],
  keys: readonly string[],
): Associations {
  const matches = new Map<string, readonly PullRequest[]>();
  const blocked = new Set<string>();
  const observations: Observation[] = [];
  for (const pr of prs) {
    const identifiers = correlate(pr, keys);
    const [identifier] = identifiers;
    if (identifiers.length !== 1 || identifier === undefined) {
      for (const id of identifiers) {
        blocked.add(id);
      }
      observations.push({
        status: identifiers.length === 0 ? "ignored" : "ambiguous",
        repository: pr.repository,
        url: pr.url,
        reason:
          identifiers.length === 0
            ? "no-configured-identifier"
            : "multiple-identifiers",
      });
    } else if (identifier.startsWith(`${pr.teamKey}-`)) {
      matches.set(identifier, [...(matches.get(identifier) ?? []), pr]);
    } else {
      blocked.add(identifier);
      observations.push({
        status: "ambiguous",
        identifier,
        url: pr.url,
        reason: "repository-team-mismatch",
      });
    }
  }
  return {
    matches: Object.fromEntries(matches),
    blocked: [...blocked],
    observations,
  };
}
async function resolvePlans(
  source: Inventory,
  links: Associations,
  providers: Providers,
): Promise<
  Readonly<{
    issues: readonly IssuePlan[];
    observations: readonly Observation[];
  }>
> {
  const issues: IssuePlan[] = [];
  const observations: Observation[] = [];
  for (const identifier of Object.keys(links.matches).toSorted()) {
    const prs = links.matches[identifier] ?? [];
    const [first] = prs;
    const team = first === undefined ? undefined : source.teams[first.teamKey];
    if (team !== undefined) {
      try {
        const issue = await providers.issue(identifier);
        if (issue.identifier !== identifier || issue.teamId !== team.id) {
          observations.push({
            status: "ambiguous",
            identifier,
            reason: "resolved-issue-team-mismatch",
          });
        } else {
          issues.push(
            planIssue(issue, {
              team,
              pullRequests: prs,
              complete:
                !links.blocked.includes(identifier) &&
                !source.blocked.includes(team.key),
            }),
          );
        }
      } catch {
        observations.push({
          status: "failed",
          identifier,
          reason: "issue-or-attachments-incomplete",
        });
      }
    }
  }
  return { issues, observations };
}
async function collect(config: Config, providers: Providers): Promise<Plan> {
  const identity = await providers.identity();
  const source = await inventory(config, providers);
  const links = associate(source.prs, [
    ...new Set(config.repositories.map((repo) => repo.teamKey)),
  ]);
  const resolved = await resolvePlans(source, links, providers);
  return {
    identity,
    issues: resolved.issues,
    observations: [
      ...source.observations,
      ...links.observations,
      ...resolved.observations,
    ],
  };
}
async function applyCompletion(
  plan: IssuePlan,
  providers: Providers,
): Promise<Observation> {
  const { identifier } = plan.issue;
  try {
    const fresh = await providers.issue(plan.issue.id);
    if (
      !sameIssueState(plan.issue, fresh) ||
      !attachmentsMatch(plan.pullRequests, fresh) ||
      plan.completeStateId === null
    ) {
      throw new Error("Issue changed since planning");
    }
    await providers.complete(plan.issue.id, plan.completeStateId);
    return {
      status: "completed",
      identifier,
      observedState: plan.completeStateId,
    };
  } catch {
    return {
      status: "failed",
      identifier,
      reason: "state-or-attachments-changed-or-mutation-or-readback-failed",
    };
  }
}
async function applyIssue(
  plan: IssuePlan,
  providers: Providers,
): Promise<Observation[]> {
  const observations: Observation[] = [];
  const { identifier } = plan.issue;
  let attachmentFailed = false;
  for (const action of plan.attachments) {
    try {
      await providers.attach(plan.issue.id, action);
      observations.push({
        status: action.operation === "link" ? "linked" : "updated",
        identifier,
        url: action.url,
        observedState: plan.issue.state.name,
      });
    } catch {
      attachmentFailed = true;
      observations.push({
        status: "failed",
        identifier,
        url: action.url,
        reason: "attachment-mutation-or-readback-failed",
      });
    }
  }
  if (plan.completeStateId !== null && !attachmentFailed) {
    observations.push(await applyCompletion(plan, providers));
  }
  if (observations.length === 0) {
    observations.push({
      status: "unchanged",
      identifier,
      observedState: plan.issue.state.name,
    });
  }
  return observations;
}
async function synchronize(
  config: Config,
  providers: Providers,
  options: RunOptions,
): Promise<Result> {
  const plan = await collect(config, providers);
  await options.announce(plan);
  const observations: Observation[] = [...plan.observations];
  for (const issue of plan.issues) {
    for (const reason of issue.problems) {
      observations.push({
        status: "failed",
        identifier: issue.issue.identifier,
        reason,
        observedState: issue.issue.state.name,
      });
    }
    if (options.apply) {
      observations.push(...(await applyIssue(issue, providers)));
    } else {
      observations.push({
        status: "unchanged",
        identifier: issue.issue.identifier,
        observedState: issue.issue.state.name,
        reason: "inspection",
      });
    }
  }
  return {
    plan,
    observations,
    exitCode: observations.some(
      (event) => event.status === "failed" || event.status === "ambiguous",
    )
      ? 1
      : 0,
  };
}
export { collect, synchronize };
export type { Observation, Plan, Result };
