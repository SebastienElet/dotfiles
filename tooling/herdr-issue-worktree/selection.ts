import type { IssueClick } from "./click.ts";
import type { RepositoryContext } from "./repository.ts";
import { inspectRepository } from "./repository.ts";

type IssueSelection = Readonly<{
  agent: "claude" | "codex";
  click: IssueClick;
  repository: RepositoryContext;
}>;

async function selectIssueWork(
  click: IssueClick,
  ask: (question: string) => Promise<string | null>,
): Promise<IssueSelection | null> {
  const choice = await ask(
    `${JSON.stringify(click.issue.url)}\n1 Claude\n2 Codex\nEscape or empty input cancels.`,
  );
  if (choice === null || choice.trim() === "") {
    return null;
  }
  if (choice.trim() !== "1" && choice.trim() !== "2") {
    throw new Error("Select Claude or Codex");
  }
  const agent = choice.trim() === "1" ? "claude" : "codex";
  const suggestion = repositorySuggestion(click);
  const selected = await ask(
    `${suggestion.description}\nRepository absolute path; Escape cancels.`,
  );
  if (selected === null || (selected === "" && suggestion.root === null)) {
    return null;
  }
  const repository = inspectRepository(
    selected === "" ? (suggestion.root ?? "") : selected,
  );
  if (
    click.issue.repository !== null &&
    !matchesIssueRepository(repository, click)
  ) {
    throw new Error("Selected repository does not match the clicked issue");
  }
  return { agent, click, repository };
}

function repositorySuggestion(
  click: IssueClick,
): Readonly<{ description: string; root: string | null }> {
  const directory = click.context.workspace_cwd;
  if (directory === undefined || directory === null) {
    return {
      description: "No current repository is available. Empty input cancels.",
      root: null,
    };
  }
  try {
    const repository = inspectRepository(directory);
    return matchesIssueRepository(repository, click)
      ? {
          description: `Press Enter to confirm ${JSON.stringify(repository.root)}.`,
          root: repository.root,
        }
      : {
          description: `Current repository ${JSON.stringify(repository.root)} is not a verified match. Choose an absolute path; empty input cancels.`,
          root: null,
        };
  } catch (error) {
    return {
      description: `Current repository unavailable: ${JSON.stringify(error instanceof Error ? error.message : String(error))}. Empty input cancels.`,
      root: null,
    };
  }
}

function matchesIssueRepository(
  repository: RepositoryContext,
  click: IssueClick,
): boolean {
  const expected = click.issue.repository;
  return (
    expected !== null &&
    repository.remotes.some(
      ({ identity }) =>
        identity?.host === expected.host &&
        (click.issue.tracker === "github"
          ? identity.path.toLowerCase() === expected.path.toLowerCase()
          : identity.path === expected.path),
    )
  );
}

export type { IssueSelection };
export { selectIssueWork };
