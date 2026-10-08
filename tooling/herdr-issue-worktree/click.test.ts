import { expect, test } from "bun:test";
import { parseIssueClick } from "./click.ts";

test("keeps the clicked URL and workspace context instead of displayed text", () => {
  const context = {
    clicked_url: "https://github.com/fixture/project/issues/17?full=1#comment",
    focused_pane_id: "w1:p1",
    invocation_source: "link_click",
    selected_text: "#99",
    workspace_cwd: "/fixture/source",
    workspace_id: "w1",
  } as const;

  expect(parseIssueClick(JSON.stringify(context))).toEqual({
    context,
    issue: {
      identity: "https://github.com/fixture/project/issues/17",
      repository: { host: "github.com", path: "fixture/project" },
      tracker: "github",
      url: context.clicked_url,
    },
  });
});

test("recognizes nested GitLab projects without dropping the clicked URL", () => {
  const clickedUrl =
    "https://gitlab.example.test/team/group/project/-/issues/17?full=1#comment";

  expect(
    parseIssueClick(
      JSON.stringify({
        clicked_url: clickedUrl,
        invocation_source: "link_click",
      }),
    ).issue,
  ).toEqual({
    identity: "https://gitlab.example.test/team/group/project/-/issues/17",
    repository: { host: "gitlab.example.test", path: "team/group/project" },
    tracker: "gitlab",
    url: clickedUrl,
  });
});

test("keeps Linear workspace identity while the local repository remains unresolved", () => {
  const clickedUrl =
    "https://linear.app/fixture/issue/TST-482/current-title?full=1#comment";

  expect(
    parseIssueClick(
      JSON.stringify({
        clicked_url: clickedUrl,
        invocation_source: "link_click",
      }),
    ).issue,
  ).toEqual({
    identity: "https://linear.app/fixture/issue/TST-482",
    repository: null,
    tracker: "linear",
    url: clickedUrl,
  });
});

test.each([
  "http://github.com/fixture/project/issues/17",
  "https://someone@github.com/fixture/project/issues/17",
  "https://someone@gitlab.example.test:8443/team/project/-/issues/17",
  "https://linear.app:8443/fixture/issue/TST-17/title",
  "https://github.com:9443/fixture/project/issues/17",
  "https://github.com/fixture/project/pull/17",
  "https://github.com.evil.test/fixture/project/issues/17",
  "https://unknown.example.test/fixture/project/issues/17",
])("rejects unsupported or misleading URLs: %s", (clickedUrl) => {
  expect(() =>
    parseIssueClick(
      JSON.stringify({
        clicked_url: clickedUrl,
        invocation_source: "link_click",
      }),
    ),
  ).toThrow("Unsupported issue URL");
});

test.each([
  "not json",
  JSON.stringify({
    selected_text: "https://github.com/fixture/project/issues/17",
  }),
  JSON.stringify({ clicked_url: 17, invocation_source: "link_click" }),
  JSON.stringify({
    clicked_url: "https://github.com/fixture/project/issues/17",
    invocation_source: "action",
  }),
])("refuses malformed or non-click context", (contextJson) => {
  expect(() => parseIssueClick(contextJson)).toThrow();
});

test("uses one GitHub issue identity across casing aliases while retaining each original URL", () => {
  const mixed = "https://github.com/Fixture/Project/issues/17?full=1#comment";
  const lower = "https://github.com/fixture/project/issues/17";
  const parse = (clicked_url: string): ReturnType<typeof parseIssueClick> =>
    parseIssueClick(
      JSON.stringify({ clicked_url, invocation_source: "link_click" }),
    );
  expect(parse(mixed).issue.identity).toBe(parse(lower).issue.identity);
  expect(parse(mixed).issue.repository).toEqual(parse(lower).issue.repository);
  expect(parse(mixed).context.clicked_url).toBe(mixed);
  expect(parse(mixed).issue.url).toBe(mixed);
});

test.each(["8443", "9443"])(
  "retains a self-hosted GitLab authority with port %s",
  (port) => {
    const url = `https://gitlab.example.test:${port}/team/project/-/issues/17?full=1#comment`;
    const { issue } = parseIssueClick(
      JSON.stringify({ clicked_url: url, invocation_source: "link_click" }),
    );
    expect(issue).toEqual({
      identity: `https://gitlab.example.test:${port}/team/project/-/issues/17`,
      repository: { host: `gitlab.example.test:${port}`, path: "team/project" },
      tracker: "gitlab",
      url,
    });
  },
);
