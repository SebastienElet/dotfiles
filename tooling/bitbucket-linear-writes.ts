import { type Graphql, readIssue } from "./bitbucket-linear-linear.ts";
import { attachmentSchema, stateSchema } from "./bitbucket-linear-schema.ts";
import type { Providers } from "./bitbucket-linear-providers.ts";
import { z } from "zod";

function attachmentWriter(graphql: Graphql): Providers["attach"] {
  const attach: Providers["attach"] = async (issueId, action) => {
    const data = z
      .object({
        attachmentCreate: z.object({
          success: z.literal(true),
          attachment: attachmentSchema,
        }),
      })
      .parse(
        await graphql(
          "mutation Attach($input: AttachmentCreateInput!) { attachmentCreate(input:$input) { success attachment { id url title subtitle } } }",
          {
            input: {
              issueId,
              url: action.url,
              title: action.title,
              subtitle: action.subtitle,
            },
          },
        ),
      );
    const actual = data.attachmentCreate.attachment;
    if (
      actual.url !== action.url ||
      actual.title !== action.title ||
      actual.subtitle !== action.subtitle
    ) {
      throw new Error("Attachment result mismatch");
    }
    const stored = await readIssue(issueId, graphql);
    const matches = stored.attachments.filter(
      (attachment) => attachment.url === action.url,
    );
    if (
      matches.length !== 1 ||
      matches[0]?.title !== action.title ||
      matches[0]?.subtitle !== action.subtitle
    ) {
      throw new Error("Attachment read-back mismatch");
    }
  };
  return attach;
}
function completionWriter(graphql: Graphql): Providers["complete"] {
  const complete: Providers["complete"] = async (issueId, stateId) => {
    const data = z
      .object({
        issueUpdate: z.object({
          success: z.literal(true),
          issue: z.object({ id: z.uuid(), state: stateSchema }),
        }),
      })
      .parse(
        await graphql(
          "mutation Complete($id:String!, $input:IssueUpdateInput!) { issueUpdate(id:$id,input:$input) { success issue { id state { id name type } } } }",
          { id: issueId, input: { stateId } },
        ),
      );
    if (
      data.issueUpdate.issue.id !== issueId ||
      data.issueUpdate.issue.state.id !== stateId
    ) {
      throw new Error("State mutation mismatch");
    }
    const stored = await readIssue(issueId, graphql);
    if (stored.state.id !== stateId || stored.state.type !== "completed") {
      throw new Error("State read-back mismatch");
    }
  };
  return complete;
}
function createWrites(
  graphql: Graphql,
): Pick<Providers, "attach" | "complete"> {
  return {
    attach: attachmentWriter(graphql),
    complete: completionWriter(graphql),
  };
}
export { createWrites };
