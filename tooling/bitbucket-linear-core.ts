import type { Issue, PullRequest, Team } from "./bitbucket-linear-schema.ts";

type AttachmentAction = Readonly<{
  url: string;
  title: string;
  subtitle: string;
  operation: "link" | "update";
}>;
type IssuePlan = Readonly<{
  issue: Issue;
  pullRequests: readonly PullRequest[];
  attachments: readonly AttachmentAction[];
  completeStateId: string | null;
  problems: readonly string[];
}>;
type PlanningInput = Readonly<{
  team: Team;
  pullRequests: readonly PullRequest[];
  complete: boolean;
}>;
function correlate(pr: PullRequest, keys: readonly string[]): string[] {
  const expression = new RegExp(
    `(?<![A-Za-z0-9_])(?:${keys.join("|")})-[0-9]+(?![A-Za-z0-9_])`,
    "giu",
  );
  return [
    ...new Set(
      [pr.title, pr.branch, pr.description].flatMap((value) =>
        [...value.matchAll(expression)].map((match: readonly string[]) =>
          (match[0] ?? "").toUpperCase(),
        ),
      ),
    ),
  ].toSorted();
}
function attachmentActions(
  issue: Issue,
  prs: readonly PullRequest[],
): AttachmentAction[] {
  return prs.flatMap((pr): AttachmentAction[] => {
    const current = issue.attachments.find(
      (attachment) => attachment.url === pr.url,
    );
    if (current?.title === pr.title && current.subtitle === pr.state) {
      return [];
    }
    return [
      {
        url: pr.url,
        title: pr.title,
        subtitle: pr.state,
        operation: current === undefined ? "link" : "update",
      },
    ];
  });
}
function completionState(
  issue: Issue,
  input: PlanningInput,
  hasProblems: boolean,
): Readonly<{ stateId: string | null; problem: string | null }> {
  const eligible =
    issue.state.type !== "completed" &&
    issue.state.type !== "canceled" &&
    input.pullRequests.some((pr) => pr.state === "MERGED") &&
    input.pullRequests.every((pr) => pr.state !== "OPEN");
  if (!eligible || !input.complete || hasProblems) {
    return { stateId: null, problem: null };
  }
  const completed = input.team.states.filter(
    (state) =>
      state.type === "completed" &&
      (input.team.completedStateId === undefined ||
        state.id === input.team.completedStateId),
  );
  if (completed.length !== 1) {
    return { stateId: null, problem: "completed-state-unresolved" };
  }
  return { stateId: completed[0]?.id ?? null, problem: null };
}
function planIssue(issue: Issue, input: PlanningInput): IssuePlan {
  const sorted = input.pullRequests.toSorted((left, right) =>
    left.url.localeCompare(right.url, "en"),
  );
  const problems: string[] = [];
  const duplicate = sorted.some(
    (pr) =>
      issue.attachments.filter((attachment) => attachment.url === pr.url)
        .length > 1,
  );
  if (duplicate) {
    problems.push("duplicate-attachment");
  }
  if (issue.archivedAt !== null) {
    problems.push("archived-issue");
  }
  if (
    issue.state.type === "completed" &&
    sorted.some((pr) => pr.state === "OPEN")
  ) {
    problems.push("completed-with-open-pr");
  }
  const attachments =
    duplicate || issue.archivedAt !== null
      ? []
      : attachmentActions(issue, sorted);
  const completion = completionState(issue, input, problems.length > 0);
  const completeStateId = completion.stateId;
  if (completion.problem !== null) {
    problems.push(completion.problem);
  }
  if (!input.complete) {
    problems.push("incomplete-inventory");
  }
  return {
    issue,
    pullRequests: sorted,
    attachments,
    completeStateId,
    problems,
  };
}
function attachmentsMatch(
  pullRequests: readonly PullRequest[],
  issue: Issue,
): boolean {
  return pullRequests.every((pr) => {
    const matches = issue.attachments.filter(
      (attachment) => attachment.url === pr.url,
    );
    const [attachment] = matches;
    return (
      matches.length === 1 &&
      attachment?.title === pr.title &&
      attachment.subtitle === pr.state
    );
  });
}
function sameIssueState(previous: Issue, current: Issue): boolean {
  return (
    previous.id === current.id &&
    previous.identifier === current.identifier &&
    previous.teamId === current.teamId &&
    previous.state.id === current.state.id &&
    previous.archivedAt === current.archivedAt
  );
}
export { attachmentsMatch, correlate, planIssue, sameIssueState };
export type { AttachmentAction, IssuePlan };
