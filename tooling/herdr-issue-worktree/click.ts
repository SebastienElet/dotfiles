import { z } from "zod";

const clickContextSchema = z
  .looseObject({
    clicked_url: z.url(),
    focused_pane_id: z.string().min(1).nullish(),
    invocation_source: z.literal("link_click"),
    selected_text: z.string().nullish(),
    workspace_cwd: z.string().min(1).nullish(),
    workspace_id: z.string().min(1).nullish(),
  })
  .readonly();

type IssueReference = Readonly<{
  identity: string;
  repository: Readonly<{ host: string; path: string }> | null;
  tracker: "github" | "gitlab" | "linear";
  url: string;
}>;

type IssueClick = Readonly<{
  context: z.infer<typeof clickContextSchema>;
  issue: IssueReference;
}>;

function parseIssueClick(contextJson: string): IssueClick {
  const context = clickContextSchema.parse(JSON.parse(contextJson));
  const clickedUrl = new URL(context.clicked_url);
  if (
    clickedUrl.protocol !== "https:" ||
    clickedUrl.username !== "" ||
    clickedUrl.password !== "" ||
    clickedUrl.port !== ""
  ) {
    throw new Error("Unsupported issue URL");
  }
  return { context, issue: recognizeIssue(context.clicked_url, clickedUrl) };
}

function recognizeIssue(
  url: string,
  clickedUrl: Readonly<Pick<URL, "hostname" | "origin" | "pathname">>,
): IssueReference {
  const github = /^\/(?<repository>[^/]+\/[^/]+)\/issues\/[1-9]\d*\/?$/u.exec(
    clickedUrl.pathname,
  );
  if (
    clickedUrl.hostname === "github.com" &&
    github?.groups?.repository !== undefined
  ) {
    const repositoryPath = github.groups.repository.toLowerCase();
    return {
      identity: `${clickedUrl.origin}${clickedUrl.pathname.replace(/\/$/u, "").toLowerCase()}`,
      repository: { host: clickedUrl.hostname, path: repositoryPath },
      tracker: "github",
      url,
    };
  }
  const gitlab =
    /^\/(?<repository>[^/]+(?:\/[^/]+)+)\/-\/issues\/[1-9]\d*\/?$/u.exec(
      clickedUrl.pathname,
    );
  if (gitlab?.groups?.repository !== undefined) {
    return {
      identity: `${clickedUrl.origin}${clickedUrl.pathname.replace(/\/$/u, "")}`,
      repository: { host: clickedUrl.hostname, path: gitlab.groups.repository },
      tracker: "gitlab",
      url,
    };
  }
  const linear =
    /^\/(?<workspace>[^/]+)\/issue\/(?<identifier>[A-Z][A-Z0-9]*-[1-9]\d*)(?:\/[^/]+)?\/?$/u.exec(
      clickedUrl.pathname,
    );
  if (
    clickedUrl.hostname === "linear.app" &&
    linear?.groups?.workspace !== undefined &&
    linear.groups.identifier !== undefined
  ) {
    return {
      identity: `${clickedUrl.origin}/${linear.groups.workspace}/issue/${linear.groups.identifier}`,
      repository: null,
      tracker: "linear",
      url,
    };
  }
  throw new Error("Unsupported issue URL");
}

export type { IssueClick, IssueReference };
export { parseIssueClick };
