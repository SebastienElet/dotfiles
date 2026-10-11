import {
  commitSha,
  gitOutput,
  gitPaths,
  rejectPrivatePaths,
} from "./secret-scan-git.ts";
import { z } from "zod";

const sha = z.string().regex(/^[a-f0-9]{40}$/u);

function eventRange(root: string, eventName: string, event: unknown): string {
  if (eventName === "pull_request") {
    const data = z
      .object({
        pull_request: z.object({
          base: z.object({ sha }),
          head: z.object({ sha }),
        }),
      })
      .parse(event);
    const head = commitSha(root, data.pull_request.head.sha);
    gitOutput(root, ["merge-base", "--is-ancestor", head, "HEAD"]);
    return `${commitSha(root, data.pull_request.base.sha)}..${head}`;
  }
  if (eventName === "push") {
    const data = z.object({ before: sha, after: sha }).parse(event);
    const head = commitSha(root, data.after);
    verifyCheckedHead(root, head);
    return /^0+$/u.test(data.before)
      ? "--all"
      : `${commitSha(root, data.before)}..${head}`;
  }
  if (eventName === "merge_group") {
    const data = z
      .object({ merge_group: z.object({ base_sha: sha, head_sha: sha }) })
      .parse(event);
    const head = commitSha(root, data.merge_group.head_sha);
    verifyCheckedHead(root, head);
    return `${commitSha(root, data.merge_group.base_sha)}..${head}`;
  }
  return "--all";
}

function verifyCheckedHead(root: string, head: string): void {
  if (head !== commitSha(root, "HEAD")) {
    throw new Error(
      "Secret scan: event head differs from the checked-out commit.",
    );
  }
}

async function introducedRange(root: string): Promise<string> {
  if (
    gitOutput(root, ["rev-parse", "--is-shallow-repository"])
      .toString()
      .trim() !== "false"
  ) {
    throw new Error("Secret scan: CI needs a complete Git checkout.");
  }
  const eventName = z
    .enum(["pull_request", "push", "merge_group", "workflow_dispatch"])
    .parse(process.env.GITHUB_EVENT_NAME);
  const eventPath = z.string().min(1).parse(process.env.GITHUB_EVENT_PATH);
  const event: unknown = await Bun.file(eventPath)
    .json()
    .catch(() => {
      throw new Error("Secret scan: unreadable GitHub event.");
    });
  return eventRange(root, eventName, event);
}

function rejectIntroducedPaths(root: string, range: string): void {
  const commits = gitOutput(root, ["rev-list", range]).toString().trim();
  for (const commit of commits === "" ? [] : commits.split("\n")) {
    rejectPrivatePaths(
      gitPaths(root, [
        "diff-tree",
        "--root",
        "-m",
        "--no-commit-id",
        "-r",
        "--name-only",
        "-z",
        "--diff-filter=ACMRT",
        commit,
      ]),
    );
  }
}

export { introducedRange, rejectIntroducedPaths };
